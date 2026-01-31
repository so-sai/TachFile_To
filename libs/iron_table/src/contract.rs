use serde::{Deserialize, Serialize};
use chrono::NaiveDate;
use std::path::PathBuf;
use thiserror::Error;

/// The foundational truth structure for a table.
/// Defines structural truth, not business truth.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableTruth {
    // Identity
    pub table_id: String,           // SHA-256(source_file + page + bbox)
    pub source_file: PathBuf,
    pub source_page: u32,

    // Structure
    pub schema: TableSchema,
    pub rows: Vec<TableRow>,

    // Provenance
    pub extraction_meta: ExtractionMeta,
    pub bbox: BoundingBox,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableSchema {
    pub columns: Vec<ColumnDef>,
    pub row_count: usize,
    pub col_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColumnDef {
    pub name: String,               // Normalized header
    pub dtype: DataType,            // Int | Float64 | Utf8 | Date
    pub unit: Option<String>,       // "m²", "VND", etc.
    pub nullable: bool,
    pub is_critical: bool,          // If true, even 'Suspicious' encoding triggers rejection
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataType {
    Int,
    Float64,
    Utf8,
    Date,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub enum EncodingStatus {
    #[default]
    Clean,
    Suspicious,
    Invalid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableRow {
    pub row_idx: usize,
    pub cells: Vec<TableCell>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableCell {
    pub row_idx: usize,
    pub col_idx: usize,
    pub value: CellValue,
    pub bbox: BoundingBox,         // Original coordinates in PDF
    pub confidence: f32,            // Docling extraction confidence
    pub source_text: String,        // Raw OCR text before normalization
    pub encoding_status: EncodingStatus,
    pub encoding_evidence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CellValue {
    Int(i64),
    Float(f64),
    Text(String),
    Date(NaiveDate),
    Null,
}

impl CellValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            CellValue::Text(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            CellValue::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            CellValue::Float(f) => Some(*f),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoundingBox {
    pub x: f32,                     // Left edge (PDF coordinates)
    pub y: f32,                     // Top edge
    pub width: f32,
    pub height: f32,
    pub page: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractionMeta {
    pub tool_version: String,
    pub timestamp: String,
    pub confidence_score: f32,
}

#[derive(Error, Debug)]
pub enum TableRejection {
    #[error("Structural Ambiguity: {0}")]
    AmbiguousStructure(String),

    #[error("Contract Violation: {0}")]
    ContractViolation(String),

    #[error("Size Constraint Violation: {0}")]
    SizeConstraintViolation(String),

    #[error("Encoding Corruption: {0}")]
    EncodingCorruption(String),

    #[error("Low Confidence: {0}")]
    LowConfidence(String),
}

/// Reasons for rejecting a specific data point or structure.
/// Aligned with TACHFILETO UI SPECIFICATION v1.0
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RejectionReason {
    EncodingCorruption,
    AmbiguousUnit,
    StructuralGhost,
    NumericOverflow,
    InvalidFormat,
}

impl TableTruth {
    pub fn validate_contract(&self) -> Result<(), TableRejection> {
        // 3.4 Rejection Rules
        
        // 1. Size Constraints
        if self.schema.row_count > 10000 {
            return Err(TableRejection::SizeConstraintViolation(format!("Row count {} > 10000", self.schema.row_count)));
        }
        if self.schema.col_count > 100 {
            return Err(TableRejection::SizeConstraintViolation(format!("Column count {} > 100", self.schema.col_count)));
        }
        if self.schema.row_count < 2 {
             return Err(TableRejection::SizeConstraintViolation(format!("Row count {} < 2", self.schema.row_count)));
        }

        // 2. Column match
        if self.schema.columns.len() != self.schema.col_count {
             return Err(TableRejection::ContractViolation(format!("Schema column count {} != defined columns {}", self.schema.col_count, self.schema.columns.len())));
        }

        // 3. Truth Scan
        for row in &self.rows {
             if row.cells.len() != self.schema.col_count {
                  return Err(TableRejection::ContractViolation(format!("Row {} cell count {} != schema col count {}", row.row_idx, row.cells.len(), self.schema.col_count)));
             }

             for cell in &row.cells {
                  let col_def = &self.schema.columns[cell.col_idx];

                  // A. Confidence Check (Skip if Null)
                  if !matches!(cell.value, CellValue::Null) && cell.confidence < 0.7 {
                       return Err(TableRejection::LowConfidence(format!("Row {}, Col {} has low confidence {}", cell.row_idx, cell.col_idx, cell.confidence)));
                  }

                  // B. Encoding Enforcement (Mission 021/022)
                  match cell.encoding_status {
                      EncodingStatus::Invalid => {
                          return Err(TableRejection::EncodingCorruption(format!(
                              "Invalid encoding at row {}, col {}: {}", 
                              cell.row_idx, cell.col_idx, cell.encoding_evidence.as_deref().unwrap_or("Unknown")
                          )));
                      }
                      EncodingStatus::Suspicious if col_def.is_critical => {
                          return Err(TableRejection::EncodingCorruption(format!(
                              "Suspicious encoding in critical column '{}' at row {}, col {}: {}", 
                              col_def.name, cell.row_idx, cell.col_idx, cell.encoding_evidence.as_deref().unwrap_or("Unknown")
                          )));
                      }
                      _ => {}
                  }
             }
        }

        Ok(())
    }
}

// ============================================================================
// PROJECT TRUTH (DERIVED LAYER)
// ============================================================================

/// ProjectTruth: Aggregated truth derived from validated TableTruth.
/// This is the output contract for Dashboard UI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectTruth {
    pub project_name: String,
    pub last_updated: String,       // ISO 8601
    pub data_source: String,

    pub project_status: ProjectStatus,
    pub status_reason: String,

    pub financials: Financials,
    pub deviation: DeviationSummary,
    pub top_risks: Vec<RiskItem>,
    pub pending_actions: Vec<ActionItem>,
    pub metrics: SystemMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ProjectStatus {
    Safe,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Financials {
    pub total_cost: f64,
    pub total_paid: f64,
    pub remaining: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviationSummary {
    pub percentage: f64,
    pub absolute: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RiskItem {
    pub description: String,
    pub severity: u8,               // 1-10
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionItem {
    pub description: String,
    pub deadline: Option<String>,   // ISO 8601
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemMetrics {
    pub table_count: usize,
    pub row_count: usize,
    pub processing_time_ms: u64,
}

