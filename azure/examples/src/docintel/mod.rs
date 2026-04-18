//! Document Intelligence

pub mod table_extract;

use eyre::{Context, Result, bail, eyre};
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::Deserialize;
use std::{path::Path, time::Duration};
use tokio::{fs, time::sleep};

const API_VERSION: &str = "2024-11-30";
const MODEL_ID: &str = "prebuilt-layout";

#[derive(Debug, Deserialize)]
struct AnalyzePollResponse {
	status: String,
	#[serde(rename = "analyzeResult")]
	analyze_result: Option<AnalyzeResult>,
}

#[derive(Debug, Deserialize)]
pub struct AnalyzeResult {
	#[serde(default)]
	pub tables: Vec<DocumentTable>,
}

#[derive(Debug, Deserialize)]
pub struct DocumentTable {
	#[serde(rename = "rowCount")]
	pub row_count: usize,
	#[serde(rename = "columnCount")]
	pub column_count: usize,
	#[serde(default)]
	pub cells: Vec<DocumentTableCell>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DocumentTableCell {
	#[serde(rename = "rowIndex")]
	pub row_index: usize,
	#[serde(rename = "columnIndex")]
	pub column_index: usize,
	pub content: String,
	#[serde(default)]
	pub kind: Option<String>, // e.g. "columnHeader"
}

pub struct AzureDocIntelClient {
	client: reqwest::Client,
	endpoint: String,
	api_key: String,
}

impl AzureDocIntelClient {
	pub fn from_env() -> Result<Self> {
		let endpoint =
			std::env::var("AZURE_DOCINTEL_ENDPOINT").context("Missing AZURE_DOCINTEL_ENDPOINT")?;
		let api_key = std::env::var("AZURE_DOCINTEL_KEY").context("Missing AZURE_DOCINTEL_KEY")?;

		Ok(Self { client: reqwest::Client::new(), endpoint, api_key })
	}

	pub async fn analyze_layout_from_path(
		&self,
		image_path: impl AsRef<Path>,
	) -> Result<AnalyzeResult> {
		let image_path = image_path.as_ref();
		let mime_type = Self::mime_type_from_path(image_path);
		let image_bytes = fs::read(image_path)
			.await
			.with_context(|| format!("Failed to read image file: {}", image_path.display()))?;
		self.analyze_layout_from_bytes(image_bytes, mime_type).await
	}

	pub async fn analyze_layout_from_bytes(
		&self,
		image_bytes: Vec<u8>,
		mime_type: &str,
	) -> Result<AnalyzeResult> {
		let poll_url = self.submit_layout_analysis(image_bytes, mime_type).await?;
		self.poll_analysis_result(&poll_url).await
	}

	pub async fn submit_layout_analysis(&self, bytes: Vec<u8>, mime_type: &str) -> Result<String> {
		let url = format!(
			"{}/documentintelligence/documentModels/{}:analyze?_overload=analyzeDocument&api-version={}",
			self.endpoint.trim_end_matches('/'),
			MODEL_ID,
			API_VERSION
		);
		let mut headers = HeaderMap::new();
		headers.insert("Ocp-Apim-Subscription-Key", HeaderValue::from_str(&self.api_key)?);
		headers.insert(CONTENT_TYPE, HeaderValue::from_str(mime_type)?);
		let resp = self
			.client
			.post(url)
			.headers(headers)
			.body(bytes)
			.send()
			.await?
			.error_for_status()?;
		let operation_location = resp
			.headers()
			.get("Operation-Location")
			.ok_or_else(|| eyre!("Missing Operation-Location header"))?
			.to_str()?
			.to_string();
		Ok(operation_location)
	}

	pub async fn poll_analysis_result(&self, poll_url: &str) -> Result<AnalyzeResult> {
		loop {
			let resp = self
				.client
				.get(poll_url)
				.header("Ocp-Apim-Subscription-Key", &self.api_key)
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

	/// Mime type from image path.
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

	/// Mime type from image bytes.
	pub fn mime_type_from_bytes(bytes: &[u8]) -> &'static str {
		if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
			"image/png"
		} else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
			"image/jpeg"
		} else if bytes.starts_with(b"BM") {
			"image/bmp"
		} else if bytes.len() >= 4 &&
			((bytes[0..4] == [0x49, 0x49, 0x2A, 0x00]) ||
				(bytes[0..4] == [0x4D, 0x4D, 0x00, 0x2A]))
		{
			"image/tiff"
		} else {
			"application/octet-stream"
		}
	}
}
