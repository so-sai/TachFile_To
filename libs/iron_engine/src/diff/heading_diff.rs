use super::report::{Delta, DeltaType};
use crate::ast::node::StableId;
use std::collections::HashSet;

/// Heading-level diff using StableId set comparison.
///
/// **Design decision**: We compare by StableId set, NOT by sequential index.
/// This means a heading reorder alone does NOT trigger a delta.
/// If position-change detection is needed in future, a separate `PositionShift`
/// DeltaType should be added — not conflated with Added/Removed.
pub struct HeadingDiffer;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HeadingEntry {
    pub id: StableId,
    pub level: u8,
}

impl HeadingDiffer {
    /// Diffs two sets of heading entries.
    /// Returns Added/Removed/StructuralChange (level change) deltas.
    pub fn diff(old: &[HeadingEntry], new: &[HeadingEntry]) -> Vec<Delta> {
        let old_map: std::collections::HashMap<u64, u8> =
            old.iter().map(|h| (h.id.0, h.level)).collect();
        let new_map: std::collections::HashMap<u64, u8> =
            new.iter().map(|h| (h.id.0, h.level)).collect();

        let mut deltas = Vec::new();

        // Check all old headings
        for (&id, &old_level) in &old_map {
            match new_map.get(&id) {
                Some(&new_level) => {
                    if old_level != new_level {
                        // Structural change: same content, different heading level
                        deltas.push(Delta {
                            node_id: StableId(id),
                            location: format!("Heading {:x}", id),
                            delta_type: DeltaType::StructuralChange {
                                description: format!(
                                    "Heading level changed: H{} → H{}",
                                    old_level, new_level
                                ),
                            },
                        });
                    }
                }
                None => {
                    deltas.push(Delta {
                        node_id: StableId(id),
                        location: format!("Heading {:x}", id),
                        delta_type: DeltaType::Removed,
                    });
                }
            }
        }

        // Find newly added headings
        for (&id, _) in &new_map {
            if !old_map.contains_key(&id) {
                deltas.push(Delta {
                    node_id: StableId(id),
                    location: format!("Heading {:x} [NEW]", id),
                    delta_type: DeltaType::Added,
                });
            }
        }

        deltas
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heading_diff_identical() {
        let headings = vec![
            HeadingEntry {
                id: StableId(10),
                level: 1,
            },
            HeadingEntry {
                id: StableId(20),
                level: 2,
            },
        ];
        let deltas = HeadingDiffer::diff(&headings, &headings);
        assert!(
            deltas.is_empty(),
            "Identical headings must produce zero deltas"
        );
    }

    #[test]
    fn test_heading_diff_level_change() {
        let old = vec![HeadingEntry {
            id: StableId(10),
            level: 2,
        }];
        let new = vec![HeadingEntry {
            id: StableId(10),
            level: 3,
        }]; // H2 → H3
        let deltas = HeadingDiffer::diff(&old, &new);
        assert_eq!(deltas.len(), 1);
        assert!(matches!(
            &deltas[0].delta_type,
            DeltaType::StructuralChange { description } if description.contains("H2 → H3")
        ));
    }

    #[test]
    fn test_heading_diff_reorder_is_silent() {
        // Reordering headings does NOT trigger a delta (set-based, not index-based)
        let a = vec![
            HeadingEntry {
                id: StableId(1),
                level: 1,
            },
            HeadingEntry {
                id: StableId(2),
                level: 1,
            },
        ];
        let b = vec![
            HeadingEntry {
                id: StableId(2),
                level: 1,
            },
            HeadingEntry {
                id: StableId(1),
                level: 1,
            },
        ];
        let deltas = HeadingDiffer::diff(&a, &b);
        assert!(
            deltas.is_empty(),
            "Reordering headings must NOT produce deltas"
        );
    }
}
