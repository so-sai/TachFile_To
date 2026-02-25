use super::report::{Delta, DeltaType};
use crate::ast::node::StableId;
use std::collections::HashSet;

/// Section-level diff using StableId set operations.
/// O(n): one pass to build each set, one pass to compute symmetric difference.
pub struct SectionDiffer;

impl SectionDiffer {
    /// Diffs two sets of section StableIds.
    /// Returns Added/Removed deltas for sections that exist on only one side.
    pub fn diff(old_ids: &[StableId], new_ids: &[StableId]) -> Vec<Delta> {
        let old_set: HashSet<u64> = old_ids.iter().map(|s| s.0).collect();
        let new_set: HashSet<u64> = new_ids.iter().map(|s| s.0).collect();

        let mut deltas = Vec::new();

        // In old, not in new → Removed
        for &id in old_set.difference(&new_set) {
            deltas.push(Delta {
                node_id: StableId(id),
                location: format!("Section {:x}", id),
                delta_type: DeltaType::Removed,
            });
        }

        // In new, not in old → Added
        for &id in new_set.difference(&old_set) {
            deltas.push(Delta {
                node_id: StableId(id),
                location: format!("Section {:x} [NEW]", id),
                delta_type: DeltaType::Added,
            });
        }

        deltas
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_diff_identical() {
        let ids = vec![StableId(1), StableId(2), StableId(3)];
        let deltas = SectionDiffer::diff(&ids, &ids);
        assert!(
            deltas.is_empty(),
            "Identical section sets must produce zero deltas"
        );
    }

    #[test]
    fn test_section_diff_added_removed() {
        let old = vec![StableId(1), StableId(2)];
        let new = vec![StableId(2), StableId(3)];
        let deltas = SectionDiffer::diff(&old, &new);

        let removed: Vec<_> = deltas
            .iter()
            .filter(|d| d.delta_type == DeltaType::Removed)
            .collect();
        let added: Vec<_> = deltas
            .iter()
            .filter(|d| d.delta_type == DeltaType::Added)
            .collect();

        assert_eq!(removed.len(), 1, "Section 1 should be Removed");
        assert_eq!(added.len(), 1, "Section 3 should be Added");
        assert_eq!(removed[0].node_id.0, 1);
        assert_eq!(added[0].node_id.0, 3);
    }
}
