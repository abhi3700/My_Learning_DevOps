//! Extract payroll table from image.

use crate::payment_table::extract_payment_rows;
use az_examples::docintel::AzureDocIntelClient;
use eyre::Result;

const DEFAULT_IMAGE_PATH: &str = "azure/examples/data/payroll.png";

#[tokio::main]
async fn main() -> Result<()> {
	dotenvy::dotenv().ok();

	let docintel = AzureDocIntelClient::from_env()?;
	let analyze_result = docintel.analyze_layout_from_path(DEFAULT_IMAGE_PATH).await?;
	let rows = extract_payment_rows(&analyze_result)?;

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

/// NOTE: In future, if there are more examples on payroll, then move this table to inside
/// `docintel/`.
mod payment_table {
	use az_examples::docintel::{
		AnalyzeResult, table_extract::extract_table_with_required_headers,
	};
	use eyre::{Result, bail};
	use std::collections::HashMap;

	const REQUIRED_HEADERS: &[&str] = &["network", "asset", "to", "amount"];

	#[derive(Debug)]
	pub struct PaymentRow {
		pub name: Option<String>,
		pub network: String,
		pub asset: String,
		pub to: String,
		pub amount: String,
		pub extra: HashMap<String, String>,
	}

	pub fn extract_payment_rows(result: &AnalyzeResult) -> Result<Vec<PaymentRow>> {
		let extracted = extract_table_with_required_headers(&result.tables, REQUIRED_HEADERS)?;
		extracted.rows.into_iter().map(payment_row_from_map).collect()
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
}
