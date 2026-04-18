//! This extracts table, but then it also customizes to payroll-like table.

use crate::docintel::DocumentTable;
use eyre::{Result, bail};
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug)]
pub struct ExtractedTable {
	pub headers: Option<Vec<String>>,
	pub rows: Vec<HashMap<String, String>>,
}

pub fn extract_table_with_required_headers(
	tables: &[DocumentTable],
	required_headers: &[&str],
) -> Result<ExtractedTable> {
	for table in tables {
		if let Some(extracted) = try_extract_table(table, required_headers)? {
			return Ok(extracted);
		}
	}

	bail!("No table found containing required headers: {:?}", required_headers);
}

fn try_extract_table(
	table: &DocumentTable,
	required_headers: &[&str],
) -> Result<Option<ExtractedTable>> {
	if table.row_count == 0 || table.column_count == 0 {
		return Ok(None);
	}

	let mut grid: BTreeMap<(usize, usize), String> = BTreeMap::new();
	let mut header_row_candidates: HashSet<usize> = HashSet::new();

	for cell in &table.cells {
		grid.insert((cell.row_index, cell.column_index), normalize_cell_text(&cell.content));

		if matches!(cell.kind.as_deref(), Some("columnHeader")) {
			header_row_candidates.insert(cell.row_index);
		}
	}

	let header_row_index = header_row_candidates.iter().min().copied().unwrap_or(0);

	let headers: Vec<String> = (0..table.column_count)
		.map(|col| grid.get(&(header_row_index, col)).cloned().unwrap_or_default())
		.collect();

	let normalized_headers: Vec<String> = headers.iter().map(|h| normalize_header(h)).collect();

	let required: HashSet<&str> = required_headers.iter().copied().collect();
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

	Ok(Some(ExtractedTable { headers: Some(normalized_headers), rows }))
}

/// Lowercase + trim, so " Network " -> "network"
pub fn normalize_header(s: &str) -> String {
	match s.trim().to_ascii_lowercase().as_str() {
		"recipient" | "receiver" | "wallet" | "wallet address" | "address" => "to".to_string(),
		"token" | "coin" => "asset".to_string(),
		"chain" | "blockchain" => "network".to_string(),
		other => other.to_string(),
	}
}

/// Minimal cleanup for OCR'd cell text
pub fn normalize_cell_text(s: &str) -> String {
	s.split_whitespace().collect::<Vec<_>>().join(" ").trim().to_string()
}
