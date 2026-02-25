//! Iron Engine: Deterministic Execution Layer (v1.0.0-core+)
//!
//! # Public API Contract
//!
//! **For Tauri/UI consumers**, use ONLY these entry points:
//! - [`DocumentSummary`] — Opaque index container
//! - [`compare_documents()`] — Deterministic diff
//! - [`DiffReport`] / [`Delta`] / [`DeltaType`] — Structured results
//!
//! **Do NOT** directly access `ast`, `diff`, or `ingestor` internals
//! from the UI layer. These modules are `pub` only for integration
//! testing within this workspace.
//!
//! **Constitutional Rules:**
//! - No schema inference
//! - No fuzzy matching
//! - No business logic
//! - Pure execution

// Internal modules — pub for workspace integration tests only.
// UI layer must use the facade below.
pub mod ast;
pub mod calculator;
pub mod diff;
pub mod exporter;
pub mod ingestor;
pub mod transformer;
pub mod validation_engine;

// ════════════════════════════════════════════════════════════════
// 1. THE PUBLIC FACADE (Stable Contract for Tauri / UI)
// ════════════════════════════════════════════════════════════════

pub use diff::report::{Delta, DeltaType, DiffReport, DiffSummary};
pub use diff::heading_diff::HeadingEntry;
pub use ingestor::StreamingIngestor;
pub use ast::node::{StableId, NumericIndexEntry};

/// Opaque summary of a parsed document.
/// Contains the lightweight indexes necessary for deterministic diffing.
/// Serializable to JSON for Tauri IPC or SQLite storage.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DocumentSummary {
    pub sections: Vec<StableId>,
    pub headings: Vec<HeadingEntry>,
    pub numeric_index: Vec<NumericIndexEntry>,
}

/// The core deterministic diff entry point.
/// Accepts two document summaries and returns a structured `DiffReport`.
pub fn compare_documents(
    old_doc: &DocumentSummary,
    new_doc: &DocumentSummary,
) -> DiffReport {
    diff::diff_documents(
        &old_doc.sections,
        &new_doc.sections,
        &old_doc.headings,
        &new_doc.headings,
        &old_doc.numeric_index,
        &new_doc.numeric_index,
    )
}

// ════════════════════════════════════════════════════════════════
// 2. LEGACY POLARS / VALIDATION EXPORTS
// ════════════════════════════════════════════════════════════════

pub use ast::{
    AstSink, Cell, Node, Row, RowType, Section, TableDefinition,
};
pub use calculator::derive_project_truth;
pub use transformer::to_dataframe;
pub use validation_engine::{ValidationContext, ValidationEngine};

/// Re-export core types from iron_table
pub use iron_table::{
    ActionItem, DataVerdict, DeviationSummary, Financials, ProjectStatus, ProjectTruth, RiskItem,
    SystemMetrics, TableTruth, ViolationType,
};

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Polars error: {0}")]
    Polars(#[from] polars::error::PolarsError),

    #[error("Schema mismatch: {0}")]
    SchemaMismatch(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),
}

pub type Result<T> = std::result::Result<T, EngineError>;
