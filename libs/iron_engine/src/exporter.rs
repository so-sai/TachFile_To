//! Exporter — AST → Markdown (V1.0)
//!
//! Clean structural Markdown output. No forensic labels. No financial summaries.
//! Heading → `#`, Table → pipe table, Paragraph → plain text. Nothing else.

use crate::ast::node::{Node, NumericIndexEntry, RowType, Section};

/// Export a collection of AST sections to clean Markdown.
///
/// Used by the `lib.rs` facade in `process_document`.
pub fn export_markdown_from_sections(sections: &[Section]) -> String {
    let mut md = String::new();

    for section in sections {
        // Section heading
        let hashes = "#".repeat(section.level as usize);
        md.push_str(&format!("{} {}\n\n", hashes, section.title));

        for node in &section.nodes {
            render_node(&mut md, node);
        }
    }

    md
}

/// Render a single AST node to Markdown.
fn render_node(md: &mut String, node: &Node) {
    match node {
        Node::Heading { level, text, .. } => {
            let hashes = "#".repeat(*level as usize);
            md.push_str(&format!("{} {}\n\n", hashes, text));
        }
        Node::Paragraph { text, .. } => {
            if !text.trim().is_empty() {
                md.push_str(&format!("{}\n\n", text.trim()));
            }
        }
        Node::Table(table_def) => {
            if table_def.rows.is_empty() {
                return;
            }

            for row in &table_def.rows {
                if row.cells.is_empty() {
                    continue;
                }
                md.push('|');
                for cell in &row.cells {
                    md.push_str(&format!(" {} |", cell.raw_text));
                }
                md.push('\n');

                // Add separator row after header
                if row.row_type == RowType::Header {
                    md.push('|');
                    for _ in &row.cells {
                        md.push_str("---|");
                    }
                    md.push('\n');
                }
            }
            md.push('\n');
        }
        Node::Fragment { .. } => {
            // Metadata markers — not rendered in Markdown output
        }
    }
}

/// Extract the numeric index from a set of sections.
/// Used by process_document to build the index for later compare operations.
pub fn extract_numeric_index(section: &Section) -> Vec<NumericIndexEntry> {
    let mut index = Vec::new();
    let section_id = section.id.0;
    let mut _table_counter: u64 = 0;

    for node in &section.nodes {
        if let Node::Table(table_def) = node {
            _table_counter += 1;
            let table_id = table_def.id.0;

            for (row_idx, row) in table_def.rows.iter().enumerate() {
                for (col_idx, cell) in row.cells.iter().enumerate() {
                    if let Some(val) = cell.numeric_value {
                        index.push(NumericIndexEntry {
                            section_id,
                            table_id,
                            row_idx,
                            col_idx,
                            numeric_value: val,
                        });
                    }
                }
            }
        }
    }

    index
}
