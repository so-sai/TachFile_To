use iron_table::contract::{TableTruth, TableRow, CellValue, DataType};
use serde::{Deserialize, Serialize};
use thiserror::Error;
// use regex::Regex;
// use std::sync::OnceLock;

#[derive(Error, Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum JanitorError {
    #[error("Uncleanable Garbage: {0}")]
    UncleanableGarbage(String),
    #[error("Type Mismatch: Expected {expected} but found {found}")]
    TypeMismatch { expected: String, found: String },
    #[error("Encoding Corruption detected: {0}")]
    EncodingCorruption(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct JanitorReport {
    pub total_cells_cleaned: usize,
    pub total_rows_ejected: usize,
    pub changes: Vec<CellChange>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CellChange {
    pub row_idx: usize,
    pub col_idx: usize,
    pub original_text: String,
    pub cleaned_value: String,
    pub action: String,
}

pub trait Janitor {
    /// Cleanses the table data. Returns a cleaned copy and a report.
    fn cleanse(&self, table: &TableTruth) -> (TableTruth, JanitorReport);
}

pub struct IronJanitor;

impl IronJanitor {
    /// The "Unit Stripper" logic.
    /// Recovers numeric values from messy strings.
    pub fn strip_units(input: &str) -> Result<f64, JanitorError> {
        let input_lower = input.to_lowercase();
        // General context flags
        let has_context = input.chars().any(|c| c.is_alphabetic()) 
            || input_lower.contains("vnd") || input_lower.contains("đ") || input_lower.contains("m3") || input_lower.contains("m2");

        // 1. Pre-clean: remove known unit suffixes that contain digits (like m3, m2)
        // This is critical so the '3' or '2' doesn't get appended to our number.
        let working_input = input_lower.replace("m3", " ").replace("m2", " ").replace("vnđ", " ").replace("vnd", " ");

        // 2. Clean the string to only digits, dots, commas, and minus sign
        let mut cleaned = String::new();
        for c in working_input.chars() {
            if c.is_ascii_digit() || c == '.' || c == ',' || c == '-' {
                cleaned.push(c);
            }
        }

        if cleaned.is_empty() || !cleaned.chars().any(|c| c.is_ascii_digit()) {
            return Err(JanitorError::UncleanableGarbage(format!("No digits found in '{}'", input)));
        }

        // 3. Identify and Normalize
        let dots = cleaned.matches('.').count();
        let commas = cleaned.matches(',').count();
        let last_dot = cleaned.rfind('.');
        let last_comma = cleaned.rfind(',');

        let normalized = match (dots, commas, last_dot, last_comma) {
            // Priority 1: Clear VN (e.g., 1.250.000,50)
            (d, 1, Some(dot_idx), Some(comma_idx)) if comma_idx > dot_idx => {
                cleaned.replace('.', "").replace(',', ".")
            }
            // Priority 2: Clear Standard (e.g., 1,250,000.50)
            (1, c, Some(dot_idx), Some(comma_idx)) if dot_idx > comma_idx => {
                cleaned.replace(',', "").to_string()
            }
            // Priority 3: Only dots
            (d, 0, Some(dot_idx), None) => {
                // If it looks like a thousand-separator block (.000) and we have context OR multiple dots
                if (cleaned.len() - dot_idx == 4) && (has_context || d > 1) {
                    cleaned.replace('.', "")
                } else {
                    cleaned
                }
            }
            // Priority 4: Only commas
            (0, c, None, Some(comma_idx)) => {
                if (cleaned.len() - comma_idx == 4) && (has_context || c > 1) {
                    cleaned.replace(',', "")
                } else {
                    cleaned.replace(',', ".")
                }
            }
            // Fallback: Remove all commas, keep dots as decimals
            _ => cleaned.replace(',', "")
        };

        normalized.parse::<f64>().map_err(|_| JanitorError::UncleanableGarbage(format!("Failed to parse normalized '{}' from original '{}'", normalized, input)))
    }

    /// The "Ghost Ejector" logic.
    fn is_ghost_row(row: &TableRow) -> bool {
        // A row is a "ghost" if:
        // 1. All cells are Null or empty
        // 2. OR overall confidence is too low (< 0.1)
        row.cells.iter().all(|c| {
            match &c.value {
                CellValue::Null => true,
                CellValue::Text(s) if s.trim().is_empty() => true,
                _ => false
            }
        }) || row.cells.iter().map(|c| c.confidence).sum::<f32>() / (row.cells.len() as f32) < 0.1
    }
}

impl Janitor for IronJanitor {
    fn cleanse(&self, table: &TableTruth) -> (TableTruth, JanitorReport) {
        let mut report = JanitorReport::default();
        let mut cleaned_rows = Vec::new();

        for row in &table.rows {
            if Self::is_ghost_row(row) {
                report.total_rows_ejected += 1;
                continue;
            }

            let mut cleaned_cells = Vec::new();
            for cell in &row.cells {
                let col_def = &table.schema.columns[cell.col_idx];
                let mut current_cell = cell.clone();

                // 1. Check for Encoding Integrity (Mission 021)
                if let CellValue::Text(s) = &cell.value {
                    match crate::gatekeeper::EncodingGatekeeper::scan(s) {
                        crate::gatekeeper::EncodingStatus::Invalid | crate::gatekeeper::EncodingStatus::Suspicious => {
                             report.changes.push(CellChange {
                                row_idx: cell.row_idx,
                                col_idx: cell.col_idx,
                                original_text: s.clone(),
                                cleaned_value: s.clone(), // Keep as is, but flag the error
                                action: "flagged_encoding_corruption".to_string(),
                            });
                            // We don't change the value, but we could return an error if we wanted to abort.
                            // For now, we allow it to flow to Truth to be REJECTED.
                        }
                        _ => {}
                    }
                }

                match (&col_def.dtype, &cell.value) {
                    (DataType::Float64, CellValue::Text(s)) => {
                        match Self::strip_units(s) {
                            Ok(val) => {
                                current_cell.value = CellValue::Float(val);
                                report.total_cells_cleaned += 1;
                                report.changes.push(CellChange {
                                    row_idx: cell.row_idx,
                                    col_idx: cell.col_idx,
                                    original_text: s.clone(),
                                    cleaned_value: val.to_string(),
                                    action: "stripped_units".to_string(),
                                });
                            }
                            Err(_) => {
                                // If unrecoverable, leave as text (will fail validation later)
                            }
                        }
                    }
                    // Add more cleaning logic for Date, Int etc. if needed
                    _ => {}
                }
                cleaned_cells.push(current_cell);
            }
            cleaned_rows.push(TableRow {
                row_idx: row.row_idx,
                cells: cleaned_cells,
            });
        }

        let mut cleaned_table = table.clone();
        cleaned_table.rows = cleaned_rows;
        cleaned_table.schema.row_count = cleaned_table.rows.len();

        (cleaned_table, report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_units_vn() {
        let val1 = IronJanitor::strip_units("1.250,50").unwrap();
        println!("DEBUG: '1.250,50' -> result: {}, expected: 1250.5", val1);
        assert_eq!(val1, 1250.5);
        
        let val2 = IronJanitor::strip_units("1.250,50 m3").unwrap();
        println!("DEBUG: '1.250,50 m3' -> result: {}, expected: 1250.5", val2);
        assert_eq!(val2, 1250.5);
        
        let val3 = IronJanitor::strip_units("VND 500.000").unwrap();
        println!("DEBUG: 'VND 500.000' -> result: {}, expected: 500000.0", val3);
        assert_eq!(val3, 500000.0);
    }

    #[test]
    fn test_strip_units_standard() {
        let val1 = IronJanitor::strip_units("1,250.50").unwrap();
        println!("Std 1,250.50 -> {}", val1);
        assert_eq!(val1, 1250.5);

        let val2 = IronJanitor::strip_units("1250.50").unwrap();
        println!("Std 1250.50 -> {}", val2);
        assert_eq!(val2, 1250.5);
    }
}
