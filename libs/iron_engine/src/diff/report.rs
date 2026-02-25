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
    }

    /// Derived from state — no manual `is_identical` field.
    /// Prevents the inconsistency risk of a stale bool flag.
    pub fn is_identical(&self) -> bool {
        self.deltas.is_empty()
    }

    /// Counts deltas by category for a quick summary.
    pub fn summary(&self) -> DiffSummary {
        let mut added = 0usize;
        let mut removed = 0usize;
        let mut value_mismatches = 0usize;
        let mut structural = 0usize;

        for d in &self.deltas {
            match &d.delta_type {
                DeltaType::Added => added += 1,
                DeltaType::Removed => removed += 1,
                DeltaType::ValueMismatch { .. } => value_mismatches += 1,
                DeltaType::StructuralChange { .. } => structural += 1,
            }
        }

        DiffSummary {
            added,
            removed,
            value_mismatches,
            structural,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSummary {
    pub added: usize,
    pub removed: usize,
    pub value_mismatches: usize,
    pub structural: usize,
}
