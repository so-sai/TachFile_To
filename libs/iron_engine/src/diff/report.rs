#![allow(dead_code)]
use crate::ast::node::StableId;
use serde::{Deserialize, Serialize};

/// The type of change detected between two document versions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeltaType {
    /// Node exists only in the new document (B).
    Added,
    /// Node exists only in the old document (A).
    Removed,
    /// Same structural identity, but numeric value changed beyond epsilon.
    ValueMismatch { old: f64, new: f64, delta: f64 },
    /// Structural property changed (column count, heading level, is_broken flag).
    StructuralChange { description: String },
}

/// A single unit of detected difference with its location context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delta {
    /// The deterministic identity of the affected node.
    pub node_id: StableId,
    /// Human-readable location path, e.g. "Section 2 > Table 1 > Row 3, Col 2"
    pub location: String,
    /// The exact nature of the change.
    pub delta_type: DeltaType,
}

/// The final machine-first report produced by the Diff Engine.
/// Serializes directly to JSON for Tauri IPC or file storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffReport {
    /// All detected deltas. Empty means identical.
    pub deltas: Vec<Delta>,
}

impl DiffReport {
    pub fn new(deltas: Vec<Delta>) -> Self {
        Self { deltas }
    }}
