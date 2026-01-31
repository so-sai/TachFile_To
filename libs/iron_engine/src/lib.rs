//! Iron Engine: Deterministic Execution Layer
//! 
//! Mission 018: The Polars Engine
//! 
//! This crate transforms validated `TableTruth` into Polars DataFrames
//! and derives `ProjectTruth` through deterministic arithmetic.
//! 
//! **Constitutional Rules:**
//! - No schema inference
//! - No fuzzy matching
//! - No business logic
//! - Pure execution

pub mod transformer;
pub mod calculator;

pub use transformer::to_dataframe;
pub use calculator::derive_project_truth;

/// Re-export core types from iron_table
pub use iron_table::{
    TableTruth, ProjectTruth, ProjectStatus,
    Financials, DeviationSummary, RiskItem, ActionItem, SystemMetrics
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
