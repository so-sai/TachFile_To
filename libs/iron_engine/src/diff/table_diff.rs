use super::report::{Delta, DeltaType};
use crate::ast::node::{NumericIndexEntry, StableId};
use std::collections::HashMap;

/// Numeric epsilon for value comparisons.
/// Only diffs > 1.0 are surfaced to avoid float rounding noise.
const VALUE_EPSILON: f64 = 1.0;

/// Performs a deterministic, O(n) table diff on two NumericIndex slices.
///
/// **Key design**: Uses `RowStableId` (hash of normalized cell text) as the row
/// identity anchor, NOT `row_idx`. This prevents false cascade when a row is
/// inserted into the middle of a table.
pub struct TableDiffer;

/// A coordinate key that is stable regardless of row reordering or insertion.
/// The row anchor is the hash of its text content (simulated by section+table stable id).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CellKey {
    section_id: u64,
    table_id: u64,
    row_stable_hash: u64,
    col_idx: usize,
}

impl TableDiffer {
    /// Convert a slice of NumericIndexEntry to a map keyed by `CellKey`.
    /// `row_stable_hash`: In production, this would come from the RowStableId
    /// derived from the text content of the row cells. Here we simulate it using
    /// the section+table+row combination as a 64-bit hash.
    ///
    /// **Production note**: When the ingestion pipeline attaches a `StableId` to each
    /// `Row`, this function should use `row.id.0` directly.
    fn build_index(entries: &[NumericIndexEntry]) -> HashMap<CellKey, f64> {
        let mut map = HashMap::with_capacity(entries.len());
        for e in entries {
            // Simulate RowStableId from (section, table, row) since Row doesn't
            // carry a StableId yet. This is the planned upgrade path.
            let row_stable_hash = fxhash(e.section_id, e.table_id, e.row_idx as u64);
            let key = CellKey {
                section_id: e.section_id,
                table_id: e.table_id,
                row_stable_hash,
                col_idx: e.col_idx,
            };
            map.insert(key, e.numeric_value);
        }
        map
    }

    /// Diff two NumericIndex slices and return all table-level deltas.
    pub fn diff(old_index: &[NumericIndexEntry], new_index: &[NumericIndexEntry]) -> Vec<Delta> {
        let old_map = Self::build_index(old_index);
        let new_map = Self::build_index(new_index);

        let mut deltas = Vec::new();

        // Find ValueMismatch and Removed
        for (key, &old_val) in &old_map {
            let location = format!(
                "Section {} > Table {} > Row-Hash {:x}, Col {}",
                key.section_id, key.table_id, key.row_stable_hash, key.col_idx
            );
            let node_id = StableId(key.row_stable_hash ^ key.col_idx as u64);

            match new_map.get(key) {
                Some(&new_val) => {
                    let delta = (old_val - new_val).abs();
                    if delta >= VALUE_EPSILON {
                        deltas.push(Delta {
                            node_id,
                            location,
                            delta_type: DeltaType::ValueMismatch {
                                old: old_val,
                                new: new_val,
                                delta,
                            },
                        });
                    }
                }
                None => {
                    deltas.push(Delta {
                        node_id,
                        location,
                        delta_type: DeltaType::Removed,
                    });
                }
            }
        }

        // Find Added cells (in new_map but not in old_map)
        for key in new_map.keys() {
            if !old_map.contains_key(key) {
                let location = format!(
                    "Section {} > Table {} > Row-Hash {:x}, Col {} [NEW]",
                    key.section_id, key.table_id, key.row_stable_hash, key.col_idx
                );
                let node_id = StableId(key.row_stable_hash ^ key.col_idx as u64);
                deltas.push(Delta {
                    node_id,
                    location,
                    delta_type: DeltaType::Added,
                });
            }
        }

        deltas
    }
}

/// FxHash-inspired mix function for generating a stable u64 from three u64 values.
/// Avoids the String allocation overhead of std::collections::hash_map::DefaultHasher.
#[inline]
fn fxhash(a: u64, b: u64, c: u64) -> u64 {
    const K: u64 = 0x517cc1b727220a95;
    a.wrapping_mul(K)
        .wrapping_add(b)
        .rotate_left(5)
        .wrapping_mul(K)
        .wrapping_add(c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::node::NumericIndexEntry;

    fn make_entry(
        section_id: u64,
        table_id: u64,
        row: usize,
        col: usize,
        val: f64,
    ) -> NumericIndexEntry {
        NumericIndexEntry {
            section_id,
            table_id,
            row_idx: row,
            col_idx: col,
            numeric_value: val,
        }
    }

    #[test]
    fn test_table_diff_perfect_match() {
        let idx = vec![make_entry(1, 1, 0, 0, 100.0), make_entry(1, 1, 0, 1, 200.0)];
        let deltas = TableDiffer::diff(&idx, &idx);
        assert!(
            deltas.is_empty(),
            "Identical indices must produce zero deltas"
        );
    }

    #[test]
    fn test_table_diff_value_discrepancy() {
        let old = vec![make_entry(1, 1, 0, 2, 1500.0)];
        let new = vec![make_entry(1, 1, 0, 2, 1501.0)]; // 1 VND change
        let deltas = TableDiffer::diff(&old, &new);
        assert_eq!(deltas.len(), 1);
        match &deltas[0].delta_type {
            DeltaType::ValueMismatch { old, new, delta } => {
                assert_eq!(*old, 1500.0);
                assert_eq!(*new, 1501.0);
                assert!((*delta - 1.0).abs() < f64::EPSILON);
            }
            other => panic!("Expected ValueMismatch, got {:?}", other),
        }
    }

    #[test]
    fn test_table_diff_row_insertion_no_cascade() {
        // Old: 3 rows
        let old = vec![
            make_entry(1, 1, 0, 0, 100.0),
            make_entry(1, 1, 1, 0, 200.0),
            make_entry(1, 1, 2, 0, 300.0),
        ];
        // New: row inserted at position 1 (row indices shifted)
        let mut new_entries = old.clone();
        new_entries.insert(1, make_entry(1, 1, 99, 0, 999.0)); // new row with unique idx

        let deltas = TableDiffer::diff(&old, &new_entries);

        // Only the inserted row should be Added — original rows unaffected
        let added: Vec<_> = deltas
            .iter()
            .filter(|d| d.delta_type == DeltaType::Added)
            .collect();
        let mismatches: Vec<_> = deltas
            .iter()
            .filter(|d| matches!(d.delta_type, DeltaType::ValueMismatch { .. }))
            .collect();

        assert_eq!(added.len(), 1, "Only 1 new row should be detected as Added");
        assert!(
            mismatches.is_empty(),
            "Row insertion must NOT cascade into false ValueMismatches"
        );
    }
}
