use serde::{Deserialize, Serialize};
use thiserror::Error;
use iron_table::contract::{TableTruth, CellValue, DataType, EncodingStatus, RejectionReason};
use sha2::{Sha256, Digest};
use std::fmt::Write;

#[derive(Error, Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum StructuredRejection {
    #[error("Structural Ambiguity: {reason}")]
    AmbiguousStructure { 
        reason: String,
        category: RejectionReason 
    },

    #[error("Contract Violation: {reason}")]
    ContractViolation { 
        reason: String,
        category: RejectionReason 
    },

    #[error("Size Constraint Violation: {reason}")]
    SizeConstraintViolation { 
        reason: String,
        category: RejectionReason 
    },

    #[error("Encoding Corruption at Row {row}, Col {col} ({column_name}): {reason}")]
    EncodingCorruption {
        row: usize,
        col: usize,
        column_name: String,
        reason: String,
        category: RejectionReason,
    },

    #[error("Low Confidence at Row {row}, Col {col} ({column_name}): {confidence}")]
    LowConfidence {
        row: usize,
        col: usize,
        column_name: String,
        confidence: f32,
        category: RejectionReason,
    },

    #[error("Type Mismatch at Row {row}, Col {col} ({column_name}): Expected {expected:?}, found {found}")]
    TypeMismatch {
        row: usize,
        col: usize,
        column_name: String,
        expected: DataType,
        found: String,
        category: RejectionReason,
    },

    #[error("Sum Mismatch for {context}: Expected {expected}, found {found}")]
    SumMismatch {
        context: String,
        expected: f64,
        found: f64,
        category: RejectionReason,
    },

    #[error("Cross-Source Contradiction: {reason}")]
    CrossSourceContradiction {
        sources: Vec<String>,
        reason: String,
        category: RejectionReason,
    },
}

/// TruthDiff: Tracks the delta between raw extraction and final human-approved truth.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TruthDiff {
    pub table_id: String,
    pub repairs: Vec<CellRepair>,
    pub violations: Vec<StructuredRejection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CellRepair {
    pub row_idx: usize,
    pub col_idx: usize,
    pub old_value: CellValue,
    pub new_value: CellValue,
    pub reason: String,
    pub reason_code: String,        // E.g., "ENCODING_REPAIR", "MANUAL_OVERRIDE"
    pub timestamp: String,          // ISO 8601
    pub actor: String,              // System Username
    pub signature_hash: String,     // Hash of this specific repair
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HashSeal {
    pub raw_input: String,
    pub correction_batch: String,
    pub virtual_truth: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TruthSnapshot {
    pub table_id: String,
    pub hashes: HashSeal,
    pub repairs: Vec<CellRepair>,   // The batch of repairs in this snapshot
    pub audited_by: String,
    pub verdict: String,
    pub timestamp: String,
    pub parent_hash: Option<String>, // For chaining history
}

impl TruthSnapshot {
    pub fn calculate_hash<T: Serialize>(data: &T) -> String {
        let json = serde_json::to_string(data).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        let result = hasher.finalize();
        result.iter().fold(String::new(), |mut output, b| {
            let _ = write!(output, "{:02x}", b);
            output
        })
    }
}

pub struct DiagnosticEngine;

impl DiagnosticEngine {
    /// Deep scan a TableTruth to collect ALL structured violations.
    /// This is the "Tutorial" layer that explains the "Verdict" of the Law.
    pub fn diagnose(table: &TableTruth) -> Vec<StructuredRejection> {
        let mut violations = Vec::new();
        
        // Size Constraints
        if table.schema.row_count > 10000 {
            violations.push(StructuredRejection::SizeConstraintViolation { 
                reason: format!("Row count {} > 10000", table.schema.row_count),
                category: RejectionReason::InvalidFormat,
            });
        }
        if table.schema.row_count < 2 {
            violations.push(StructuredRejection::SizeConstraintViolation { 
                reason: format!("Row count {} < 2", table.schema.row_count),
                category: RejectionReason::InvalidFormat,
            });
        }

        // Truth Scan
        for row in &table.rows {
            for cell in &row.cells {
                let col_def = &table.schema.columns[cell.col_idx];

                // Confidence Check
                if !matches!(cell.value, CellValue::Null) && cell.confidence < 0.7 {
                    violations.push(StructuredRejection::LowConfidence {
                        row: cell.row_idx,
                        col: cell.col_idx,
                        column_name: col_def.name.clone(),
                        confidence: cell.confidence,
                        category: RejectionReason::InvalidFormat, // Low confidence often reflects format issues
                    });
                }

                // Encoding Check
                match cell.encoding_status {
                    EncodingStatus::Invalid => {
                        violations.push(StructuredRejection::EncodingCorruption {
                            row: cell.row_idx,
                            col: cell.col_idx,
                            column_name: col_def.name.clone(),
                            reason: cell.encoding_evidence.clone().unwrap_or_else(|| "Unknown".to_string()),
                            category: RejectionReason::EncodingCorruption,
                        });
                    }
                    EncodingStatus::Suspicious if col_def.is_critical => {
                        violations.push(StructuredRejection::EncodingCorruption {
                            row: cell.row_idx,
                            col: cell.col_idx,
                            column_name: col_def.name.clone(),
                            reason: format!("Suspicious in critical column: {}", cell.encoding_evidence.as_deref().unwrap_or("Unknown")),
                            category: RejectionReason::EncodingCorruption,
                        });
                    }
                    _ => {}
                }
            }
        }
        
        // --- MISSION 026: CROSS-SOURCE CONTRADICTION DETECTION ---
        // Scenario 1: Aggregated Sum vs Detail (e.g. Total on Summary vs sum of breakdown)
        // This is a heuristic: check if specific columns (like 'Amount') have a row that matches the sum of others.
        // If not, and they are marked as Summary vs Detail, flag it.
        // For now, we implement a simple sum-check on all numeric columns if a 'Total' row exists.
        
        let numeric_cols: Vec<usize> = table.schema.columns.iter()
            .enumerate()
            .filter(|(_, col)| col.dtype == DataType::Float64)
            .map(|(i, _)| i)
            .collect();

        for col_idx in numeric_cols {
            let mut sum = 0.0;
            let mut total_found = None;

            for (row_idx, row) in table.rows.iter().enumerate() {
                if let Some(cell) = row.cells.get(col_idx) {
                    if let CellValue::Float(val) = cell.value {
                        // Heuristic: If row contains 'Total' or 'Tổng' in any string column
                        let is_total_row = row.cells.iter().any(|c| {
                            if let CellValue::Text(s) = &c.value {
                                let s_lower = s.to_lowercase();
                                s_lower.contains("total") || s_lower.contains("tổng") || s_lower.contains("sum")
                            } else {
                                false
                            }
                        });

                        if is_total_row {
                            total_found = Some((row_idx, val));
                        } else {
                            sum += val;
                        }
                    }
                }
            }

            if let Some((row_idx, total_val)) = total_found {
                if (total_val - sum).abs() > 0.01 {
                    violations.push(StructuredRejection::CrossSourceContradiction {
                        sources: vec![format!("Row {}", row_idx), "Accumulated Sum".to_string()],
                        reason: format!(
                            "Numeric Discrepancy: Total ({}) does not match sum of details ({})",
                            total_val, sum
                        ),
                        category: RejectionReason::NumericOverflow, // closest fit for discrepancy
                    });
                }
            }
        }
        
        violations
    }
}
