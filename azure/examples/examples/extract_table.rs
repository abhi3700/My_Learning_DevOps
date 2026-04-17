use eyre::{Context, Result, bail, eyre};
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::Deserialize;
use std::{
	collections::{BTreeMap, HashMap, HashSet},
	path::Path,
	time::Duration,
};
use tokio::{fs, time::sleep};

const API_VERSION: &str = "2024-11-30";
const MODEL_ID: &str = "prebuilt-layout";
const REQUIRED_HEADERS: &[&str] = &["network", "asset", "to", "amount"];
const DEFAULT_IMAGE_PATH: &str = "azure/examples/data/payroll.png";

#[derive(Debug, Deserialize)]
struct AnalyzePollResponse {
	status: String,
	#[serde(rename = "analyzeResult")]
	analyze_result: Option<AnalyzeResult>,
}

#[derive(Debug, Deserialize)]
struct AnalyzeResult {
	#[serde(default)]
	tables: Vec<DocumentTable>,
}

#[derive(Debug, Deserialize)]
struct DocumentTable {
	#[serde(rename = "rowCount")]
	row_count: usize,
	#[serde(rename = "columnCount")]
	column_count: usize,
	#[serde(default)]
	cells: Vec<DocumentTableCell>,
}

#[derive(Debug, Deserialize, Clone)]
struct DocumentTableCell {
	#[serde(rename = "rowIndex")]
	row_index: usize,
	#[serde(rename = "columnIndex")]
	column_index: usize,
	content: String,
	#[serde(default)]
	kind: Option<String>, // e.g. "columnHeader"
}

#[derive(Debug)]
struct ExtractedTable {
	headers: Vec<String>,
	rows: Vec<HashMap<String, String>>,
}

#[derive(Debug)]
struct PaymentRow {
	name: Option<String>,
	network: String,
	asset: String,
	to: String,
	amount: String,
	extra: HashMap<String, String>,
}

struct AzureDocIntelClient {
	client: reqwest::Client,
	endpoint: String,
	api_key: String,
}

#[tokio::main]
async fn main() -> Result<()> {
	dotenvy::dotenv().ok();

	let docintel = AzureDocIntelClient::from_env()?;
	let rows = docintel.extract_payment_rows_from_image(DEFAULT_IMAGE_PATH).await?;

	for (idx, row) in rows.iter().enumerate() {
		println!("Row {}:", idx + 1);
		if let Some(name) = &row.name {
			println!("  name: {name}");
		}
		println!("  network: {}", row.network);
		println!("  asset: {}", row.asset);
		println!("  to: {}", row.to);
		println!("  amount: {}", row.amount);

		if !row.extra.is_empty() {
			println!("  extra:");
			for (key, value) in &row.extra {
				println!("    {key}: {value}");
			}
		}

		println!();
	}

	Ok(())
}

impl AzureDocIntelClient {
	fn from_env() -> Result<Self> {
		let endpoint =
			std::env::var("AZURE_DOCINTEL_ENDPOINT").context("Missing AZURE_DOCINTEL_ENDPOINT")?;
		let api_key = std::env::var("AZURE_DOCINTEL_KEY").context("Missing AZURE_DOCINTEL_KEY")?;

		Ok(Self { client: reqwest::Client::new(), endpoint, api_key })
	}

	async fn extract_payment_rows_from_image(
		&self,
		image_path: impl AsRef<Path>,
	) -> Result<Vec<PaymentRow>> {
		let image_path = image_path.as_ref();
		let mime_type = mime_type_from_path(image_path);
		let image_bytes = fs::read(image_path)
			.await
			.with_context(|| format!("Failed to read image file: {}", image_path.display()))?;

		let poll_url = submit_layout_analysis(
			&self.client,
			&self.endpoint,
			&self.api_key,
			image_bytes,
			mime_type,
		)
		.await?;

		let result = poll_analysis_result(&self.client, &poll_url, &self.api_key).await?;
		let extracted = extract_target_table(&result.tables)?;

		extracted.rows.into_iter().map(payment_row_from_map).collect()
	}
}

fn payment_row_from_map(mut row: HashMap<String, String>) -> Result<PaymentRow> {
	let network = take_required(&mut row, "network")?;
	let asset = take_required(&mut row, "asset")?;
	let to = take_required(&mut row, "to")?;
	let amount = take_required(&mut row, "amount")?;
	let name = row.remove("name").filter(|value| !value.is_empty());

	Ok(PaymentRow { name, network, asset, to, amount, extra: row })
}

fn take_required(row: &mut HashMap<String, String>, key: &str) -> Result<String> {
	match row.remove(key) {
		Some(value) if !value.is_empty() => Ok(value),
		_ => bail!("Missing required column value: {key}"),
	}
}

fn mime_type_from_path(path: &Path) -> &'static str {
	match path
		.extension()
		.and_then(|ext| ext.to_str())
		.map(|ext| ext.to_ascii_lowercase())
		.as_deref()
	{
		Some("png") => "image/png",
		Some("jpg") | Some("jpeg") => "image/jpeg",
		Some("bmp") => "image/bmp",
		Some("tif") | Some("tiff") => "image/tiff",
		_ => "application/octet-stream",
	}
}

async fn submit_layout_analysis(
	client: &reqwest::Client,
	endpoint: &str,
	api_key: &str,
	bytes: Vec<u8>,
	mime_type: &str,
) -> Result<String> {
	let url = format!(
		"{}/documentintelligence/documentModels/{}:analyze?_overload=analyzeDocument&api-version={}",
		endpoint.trim_end_matches('/'),
		MODEL_ID,
		API_VERSION
	);

	let mut headers = HeaderMap::new();
	headers.insert("Ocp-Apim-Subscription-Key", HeaderValue::from_str(api_key)?);
	headers.insert(CONTENT_TYPE, HeaderValue::from_str(mime_type)?);

	let resp = client.post(url).headers(headers).body(bytes).send().await?.error_for_status()?;

	let operation_location = resp
		.headers()
		.get("Operation-Location")
		.ok_or_else(|| eyre!("Missing Operation-Location header"))?
		.to_str()?
		.to_string();

	Ok(operation_location)
}

async fn poll_analysis_result(
	client: &reqwest::Client,
	poll_url: &str,
	api_key: &str,
) -> Result<AnalyzeResult> {
	loop {
		let resp = client
			.get(poll_url)
			.header("Ocp-Apim-Subscription-Key", api_key)
			.send()
			.await?
			.error_for_status()?
			.json::<AnalyzePollResponse>()
			.await?;

		match resp.status.as_str() {
			"succeeded" => {
				return resp
					.analyze_result
					.ok_or_else(|| eyre!("Analyze succeeded but analyzeResult is missing"));
			},
			"failed" => bail!("Azure analysis failed"),
			_ => sleep(Duration::from_secs(2)).await,
		}
	}
}

/// Finds the first table whose headers include:
/// network, asset, to, amount
///
/// Keeps extra columns too.
fn extract_target_table(tables: &[DocumentTable]) -> Result<ExtractedTable> {
	for table in tables {
		if let Some(extracted) = try_extract_table(table)? {
			return Ok(extracted);
		}
	}

	bail!("No table found containing required headers: {:?}", REQUIRED_HEADERS);
}

fn try_extract_table(table: &DocumentTable) -> Result<Option<ExtractedTable>> {
	if table.row_count == 0 || table.column_count == 0 {
		return Ok(None);
	}

	// Build sparse grid
	let mut grid: BTreeMap<(usize, usize), String> = BTreeMap::new();
	let mut header_row_candidates: HashSet<usize> = HashSet::new();

	for cell in &table.cells {
		grid.insert((cell.row_index, cell.column_index), normalize_cell_text(&cell.content));

		if matches!(cell.kind.as_deref(), Some("columnHeader")) {
			header_row_candidates.insert(cell.row_index);
		}
	}

	// Prefer Azure-marked header row; fallback to row 0.
	let header_row_index = if let Some(min_header_row) = header_row_candidates.iter().min() {
		*min_header_row
	} else {
		0
	};

	let headers: Vec<String> = (0..table.column_count)
		.map(|col| grid.get(&(header_row_index, col)).cloned().unwrap_or_default())
		.collect();

	let normalized_headers: Vec<String> = headers.iter().map(|h| normalize_header(h)).collect();

	let required: HashSet<&str> = REQUIRED_HEADERS.iter().copied().collect();
	let actual: HashSet<&str> = normalized_headers.iter().map(String::as_str).collect();

	if !required.is_subset(&actual) {
		return Ok(None);
	}

	let mut rows = Vec::new();

	for row_idx in (header_row_index + 1)..table.row_count {
		let mut row_map = HashMap::new();
		let mut has_any_value = false;

		for col_idx in 0..table.column_count {
			let header = normalized_headers.get(col_idx).cloned().unwrap_or_default();

			if header.is_empty() {
				continue;
			}

			let value = grid.get(&(row_idx, col_idx)).cloned().unwrap_or_default();

			if !value.is_empty() {
				has_any_value = true;
			}

			row_map.insert(header, value);
		}

		if has_any_value {
			rows.push(row_map);
		}
	}

	Ok(Some(ExtractedTable { headers: normalized_headers, rows }))
}

/// Lowercase + trim, so " Network " -> "network"
fn normalize_header(s: &str) -> String {
	match s.trim().to_ascii_lowercase().as_str() {
		"recipient" | "receiver" | "wallet" | "wallet address" | "address" => "to".to_string(),
		"token" | "coin" => "asset".to_string(),
		"chain" | "blockchain" => "network".to_string(),
		other => other.to_string(),
	}
}

/// Minimal cleanup for OCR'd cell text
fn normalize_cell_text(s: &str) -> String {
	s.split_whitespace().collect::<Vec<_>>().join(" ").trim().to_string()
}
