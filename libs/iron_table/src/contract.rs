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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataType {
    Int,
    Float64,
    Utf8,
    Date,
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

        // 3. Confidence Check
        // Flatten rows to check cells
        for row in &self.rows {
             if row.cells.len() != self.schema.col_count {
                  return Err(TableRejection::ContractViolation(format!("Row {} cell count {} != schema col count {}", row.row_idx, row.cells.len(), self.schema.col_count)));
             }
             for cell in &row.cells {
                  if cell.confidence < 0.7 {
                        return Err(TableRejection::ContractViolation(format!("Cell at ({}, {}) has low confidence {}", cell.row_idx, cell.col_idx, cell.confidence)));
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

