use crate::ast::node::{Row, RowType};

/// Represents a 2D bounding box on a page.
#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

/// A parsed text element from an OCR engine (e.g., MuPDF or Docling)
#[derive(Debug, Clone)]
pub struct TextElement {
    pub text: String,
    pub bbox: BoundingBox,
}

/// Applies the Whitespace Alignment Heuristic.
/// Processes a vector of TextElements and groups them into columns
/// by identifying vertical whitespace gaps via a histogram approach.
pub struct ColumnBoundaryDetector;

impl ColumnBoundaryDetector {
    /// Detects column boundaries (X-coordinates) based on whitespace gaps.
    pub fn detect_boundaries(elements: &[TextElement], page_width: f64) -> Vec<f64> {
        let mut histogram = vec![0; page_width as usize + 1];

        // 1. Build a density map across the X-axis
        for el in elements {
            let start = el.bbox.x0.max(0.0) as usize;
            let end = (el.bbox.x1.min(page_width) as usize).min(page_width as usize);
            for x in start..=end {
                if x < histogram.len() {
                    histogram[x] += 1;
                }
            }
        }

        // 2. Identify valleys (whitespace gaps) -> Column Boundaries
        let mut boundaries = Vec::new();
        let mut in_gap = true;
        let mut gap_start = 0;

        for (x, &density) in histogram.iter().enumerate() {
            if density == 0 {
                if !in_gap {
                    in_gap = true;
                    gap_start = x;
                }
            } else {
                if in_gap {
                    let gap_end = x;
                    let gap_width = gap_end - gap_start;
                    // If the gap is wide enough to be a column separator
                    // (heuristic: > 5 pixels, depending on scale)
                    if gap_width > 5 && gap_start > 0 {
                        boundaries.push((gap_start + gap_end) as f64 / 2.0); // Midpoint of gap
                    }
                    in_gap = false;
                }
            }
        }

        boundaries
    }
}

/// Applies the Row Cohesion Heuristic
/// Merges broken description rows into their parent row.
pub struct RowCohesionMapper;

impl RowCohesionMapper {
    /// Iterates through parsed physical rows. If row N+1 has NO numeric data
    /// and its text starts noticeably indented or just below row N, its text is appended to row N.
    pub fn merge_broken_rows(mut physical_rows: Vec<Row>) -> Vec<Row> {
        if physical_rows.is_empty() {
            return Vec::new();
        }

        let mut consolidated_rows: Vec<Row> = Vec::new();
        let mut current_parent = physical_rows.remove(0);

        for next_row in physical_rows {
            // Check if `next_row` is completely devoid of numbers
            let has_numbers = next_row.cells.iter().any(|c| c.numeric_value.is_some());

            // Check if `next_row` is primarily text (Description overflow)
            let is_text_heavy = next_row
                .cells
                .iter()
                .filter(|c| !c.raw_text.trim().is_empty())
                .all(|c| c.numeric_value.is_none());

            if !has_numbers && is_text_heavy && current_parent.row_type == RowType::Data {
                // Merge next_row into current_parent iteratively cell by cell
                for (parent_col, child_col) in
                    current_parent.cells.iter_mut().zip(next_row.cells.iter())
                {
                    let child_text = child_col.raw_text.trim();
                    if !child_text.is_empty() {
                        parent_col.raw_text.push_str(" ");
                        parent_col.raw_text.push_str(child_text);
                    }
                }
            } else {
                // Push the finalized parent and start a new one
                consolidated_rows.push(current_parent);
                current_parent = next_row;
            }
        }

        // Push the final row
        consolidated_rows.push(current_parent);

        consolidated_rows
    }
}

/// Gom các phần tử văn bản rời rạc thành các hàng logic dựa trên tọa độ Y (Geometric Snap).
pub struct RowSnapper {
    pub tolerance: f64, // Ngưỡng lệch tọa độ (thường là 30% tuyến tính line-height)
}

impl RowSnapper {
    pub fn new(tolerance: f64) -> Self {
        Self { tolerance }
    }

    /// Gom các phần tử văn bản rời rạc thành các hàng logic
    pub fn snap_to_rows(&self, mut elements: Vec<TextElement>) -> Vec<Vec<TextElement>> {
        if elements.is_empty() {
            return Vec::new();
        }

        // Helper to get center Y of a bounding box
        let get_center_y = |el: &TextElement| (el.bbox.y0 + el.bbox.y1) / 2.0;

        // Bước 1: Sắp xếp theo Y center tăng dần
        elements.sort_by(|a, b| {
            get_center_y(a)
                .partial_cmp(&get_center_y(b))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut rows: Vec<Vec<TextElement>> = Vec::new();
        let mut current_row: Vec<TextElement> = Vec::new();
        let mut current_y = get_center_y(&elements[0]);

        // Bước 2: Group by Y với ngưỡng tolerance
        for el in elements {
            let center_y = get_center_y(&el);
            if (center_y - current_y).abs() <= self.tolerance {
                current_row.push(el);
            } else {
                // Bước 3: Sắp xếp phần tử trong hàng theo X
                current_row.sort_by(|a, b| {
                    a.bbox
                        .x0
                        .partial_cmp(&b.bbox.x0)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                rows.push(current_row);

                // Start new row
                current_y = center_y;
                current_row = vec![el];
            }
        }

        if !current_row.is_empty() {
            current_row.sort_by(|a, b| {
                a.bbox
                    .x0
                    .partial_cmp(&b.bbox.x0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            rows.push(current_row);
        }

        rows
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::node::{Cell, Row, RowType};

    #[test]
    fn test_row_snapper_geometric() {
        let snapper = RowSnapper::new(5.0);

        let elements = vec![
            TextElement {
                text: "Tổng".to_string(),
                bbox: BoundingBox {
                    x0: 10.0,
                    y0: 100.0,
                    x1: 50.0,
                    y1: 110.0,
                },
            },
            TextElement {
                text: "1.000.000".to_string(),
                bbox: BoundingBox {
                    x0: 200.0,
                    y0: 102.0,
                    x1: 280.0,
                    y1: 112.0,
                },
            }, // Slightly lower
            TextElement {
                text: "Hạng mục A".to_string(),
                bbox: BoundingBox {
                    x0: 10.0,
                    y0: 150.0,
                    x1: 90.0,
                    y1: 160.0,
                },
            },
            TextElement {
                text: "2.500".to_string(),
                bbox: BoundingBox {
                    x0: 200.0,
                    y0: 148.0,
                    x1: 240.0,
                    y1: 158.0,
                },
            }, // Slightly higher
        ];

        let rows = snapper.snap_to_rows(elements);

        assert_eq!(rows.len(), 2);

        // Check Row 1
        assert_eq!(rows[0].len(), 2);
        assert_eq!(rows[0][0].text, "Tổng");
        assert_eq!(rows[0][1].text, "1.000.000");

        // Check Row 2
        assert_eq!(rows[1].len(), 2);
        assert_eq!(rows[1][0].text, "Hạng mục A");
        assert_eq!(rows[1][1].text, "2.500");
    }

    #[test]
    fn test_row_cohesion() {
        let rows = vec![
            Row {
                cells: vec![
                    Cell {
                        raw_text: "1".to_string(),
                        numeric_value: Some(1.0),
                    },
                    Cell {
                        raw_text: "Công tác đất".to_string(),
                        numeric_value: None,
                    },
                    Cell {
                        raw_text: "100".to_string(),
                        numeric_value: Some(100.0),
                    },
                ],
                row_type: RowType::Data,
            },
            Row {
                cells: vec![
                    Cell {
                        raw_text: "".to_string(),
                        numeric_value: None,
                    },
                    Cell {
                        raw_text: "- Đào móng".to_string(),
                        numeric_value: None,
                    },
                    Cell {
                        raw_text: "".to_string(),
                        numeric_value: None,
                    },
                ],
                row_type: RowType::Data,
            },
            Row {
                cells: vec![
                    Cell {
                        raw_text: "".to_string(),
                        numeric_value: None,
                    },
                    Cell {
                        raw_text: "- Đắp đất".to_string(),
                        numeric_value: None,
                    },
                    Cell {
                        raw_text: "".to_string(),
                        numeric_value: None,
                    },
                ],
                row_type: RowType::Data,
            },
        ];

        let consolidated = RowCohesionMapper::merge_broken_rows(rows);
        assert_eq!(consolidated.len(), 1);
        assert_eq!(
            consolidated[0].cells[1].raw_text,
            "Công tác đất - Đào móng - Đắp đất"
        );
    }

    #[test]
    fn test_column_boundary_detector() {
        let elements = vec![
            // Column 1
            TextElement {
                text: "A".to_string(),
                bbox: BoundingBox {
                    x0: 10.0,
                    y0: 10.0,
                    x1: 20.0,
                    y1: 20.0,
                },
            },
            TextElement {
                text: "B".to_string(),
                bbox: BoundingBox {
                    x0: 10.0,
                    y0: 30.0,
                    x1: 20.0,
                    y1: 40.0,
                },
            },
            // Gap from 20 to 50
            // Column 2
            TextElement {
                text: "C".to_string(),
                bbox: BoundingBox {
                    x0: 50.0,
                    y0: 10.0,
                    x1: 60.0,
                    y1: 20.0,
                },
            },
        ];

        let page_width = 100.0;
        let boundaries = ColumnBoundaryDetector::detect_boundaries(&elements, page_width);

        // Gaps > 5 pixels are considered boundaries.
        // Here gap is 21 to 50 = 29 wide. Start is 21. Midpoint is 35.5.
        assert_eq!(boundaries.len(), 1);
        assert!((boundaries[0] - 35.5).abs() < f64::EPSILON);
    }
}
