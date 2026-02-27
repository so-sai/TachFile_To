//! Deterministic Diff Engine
//!
//! Compares two versions of a parsed document (via their AST indices) and
//! produces a structured, machine-first `DiffReport` (JSON-serializable).
//!
//! **Algorithm complexity**: O(n) via HashMap join — no quadratic comparisons.
//! **Stability guarantee**: Uses StableId (content hash), NOT physical positions.

#![allow(unused_imports)]

pub mod heading_diff;
pub mod report;
pub mod section_diff;
pub mod table_diff;

pub use heading_diff::{HeadingDiffer, HeadingEntry};
pub use report::{Delta, DeltaType, DiffReport};
pub use section_diff::SectionDiffer;
pub use table_diff::TableDiffer;

use crate::ast::node::{NumericIndexEntry, StableId};

/// Top-level convenience function that runs all three diff passes and
/// consolidates the results into a single `DiffReport`.
pub fn diff_documents(
    old_sections: &[StableId],
    new_sections: &[StableId],
    old_headings: &[HeadingEntry],
    new_headings: &[HeadingEntry],
    old_numeric_index: &[NumericIndexEntry],
    new_numeric_index: &[NumericIndexEntry],
) -> DiffReport {
    let mut all_deltas = Vec::new();

    all_deltas.extend(SectionDiffer::diff(old_sections, new_sections));
    all_deltas.extend(HeadingDiffer::diff(old_headings, new_headings));
    all_deltas.extend(TableDiffer::diff(old_numeric_index, new_numeric_index));

    DiffReport::new(all_deltas)
}

#[cfg(test)]
mod integration_tests {
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
    fn test_diff_perfect_match_produces_no_deltas() {
        let sections = vec![StableId(1), StableId(2)];
        let headings = vec![HeadingEntry {
            id: StableId(1),
            level: 1,
        }];
        let index = vec![make_entry(1, 1, 0, 0, 1000.0)];

        let report = diff_documents(&sections, &sections, &headings, &headings, &index, &index);

        assert!(
            report.is_identical(),
            "Perfect match must produce zero deltas"
        );
        assert_eq!(report.deltas.len(), 0);
    }

    #[test]
    fn test_diff_detects_value_change() {
        let sections = vec![StableId(1)];
        let headings = vec![HeadingEntry {
            id: StableId(1),
            level: 1,
        }];
        let old_index = vec![make_entry(1, 1, 0, 0, 10_000.0)];
        let new_index = vec![make_entry(1, 1, 0, 0, 12_000.0)]; // +2000

        let report = diff_documents(
            &sections, &sections, &headings, &headings, &old_index, &new_index,
        );

        assert!(!report.is_identical());
        let summary = report.summary();
        assert_eq!(summary.value_mismatches, 1);
        assert_eq!(summary.added, 0);
        assert_eq!(summary.removed, 0);

        match &report.deltas[0].delta_type {
            DeltaType::ValueMismatch { old, new, delta } => {
                assert_eq!(*old, 10_000.0);
                assert_eq!(*new, 12_000.0);
                assert!((*delta - 2000.0).abs() < 1.0);
            }
            other => panic!("Expected ValueMismatch, got {:?}", other),
        }
    }

    #[test]
    fn test_diff_below_epsilon_is_silent() {
        // A change of 0.5 VND must NOT be reported (< 1.0 epsilon)
        let sections = vec![StableId(1)];
        let headings: Vec<HeadingEntry> = vec![];
        let old_index = vec![make_entry(1, 1, 0, 0, 1000.0)];
        let new_index = vec![make_entry(1, 1, 0, 0, 1000.5)]; // 0.5 VND noise

        let report = diff_documents(
            &sections, &sections, &headings, &headings, &old_index, &new_index,
        );

        assert!(
            report.is_identical(),
            "Changes below epsilon (0.5 VND) must be silent"
        );
    }
}
