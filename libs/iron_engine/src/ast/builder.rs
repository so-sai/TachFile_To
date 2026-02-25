use super::node::{Node, NumericIndexEntry, RowType};
use super::sink::AstSink;
use std::io::Write;

/// A concrete implementation of AstSink that builds the Markdown output stream
/// and retains specifically the numeric indices for cross-file diffing.
pub struct AstMarkdownBuilder<W: Write> {
    /// The accumulating markdown output sink (e.g. BufWriter<File>).
    pub writer: W,

    /// The lightweight numeric index retained in memory for R01/R05 rules.
    pub numeric_index: Vec<NumericIndexEntry>,

    /// The current un-flushed section being built.
    current_section_nodes: Vec<Node>,

    /// Counters for generating IDs
    pub section_counter: u64,
    pub table_counter: u64,
}

impl<W: Write> AstMarkdownBuilder<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            numeric_index: Vec::new(),
            current_section_nodes: Vec::new(),
            section_counter: 0,
            table_counter: 0,
        }
    }

    /// Internal helper to serialize a section immediately to the markdown buffer.
    fn serialize_section_to_md(
        &mut self,
        title: &str,
        level: u8,
        nodes: &[Node],
    ) -> std::io::Result<()> {
        let hashes = "#".repeat(level as usize);
        writeln!(&mut self.writer, "{} {}\n", hashes, title)?;

        for node in nodes {
            match node {
                Node::Heading { level, text, .. } => {
                    let h = "#".repeat(*level as usize);
                    writeln!(&mut self.writer, "{} {}\n", h, text)?;
                }
                Node::Paragraph { text, .. } => {
                    writeln!(&mut self.writer, "{}\n", text)?;
                }
                Node::Table(table_def) => {
                    self.table_counter += 1;
                    if table_def.rows.is_empty() {
                        continue;
                    }

                    // Simple MD Table Serialization
                    for (r_idx, row) in table_def.rows.iter().enumerate() {
                        write!(&mut self.writer, "|")?;
                        for (c_idx, cell) in row.cells.iter().enumerate() {
                            write!(&mut self.writer, " {} |", cell.raw_text)?;

                            // Capture the numeric value for the Lightweight Index
                            if let Some(val) = cell.numeric_value {
                                self.numeric_index.push(NumericIndexEntry {
                                    section_id: self.section_counter,
                                    table_id: self.table_counter,
                                    row_idx: r_idx,
                                    col_idx: c_idx,
                                    numeric_value: val,
                                });
                            }
                        }
                        writeln!(&mut self.writer)?;

                        // Add separator after header
                        if row.row_type == RowType::Header {
                            write!(&mut self.writer, "|")?;
                            for _ in &row.cells {
                                write!(&mut self.writer, "---|")?;
                            }
                            writeln!(&mut self.writer)?;
                        }
                    }
                    writeln!(&mut self.writer)?;
                }
                Node::Fragment { .. } => {
                    // Fragments are metadata markers, ignored in final markdown output
                }
            }
        }

        self.writer.flush()?;
        Ok(())
    }
}

impl<W: Write + Send> AstSink for AstMarkdownBuilder<W> {
    fn push_node(&mut self, node: Node) {
        self.current_section_nodes.push(node);
    }

    fn finalize_section(&mut self, title: String, level: u8) -> Result<(), String> {
        self.section_counter += 1;

        // Extract nodes to avoid immutable borrow overlap with &mut self
        let nodes = std::mem::take(&mut self.current_section_nodes);

        // 1. Serialize the current buffered nodes to Markdown
        if let Err(e) = self.serialize_section_to_md(&title, level, &nodes) {
            return Err(format!("IO Error during finalize_section: {}", e));
        }

        // 2. Buffer is implicitly cleared as `nodes` is dropped.

        Ok(())
    }

    fn flush(&mut self) -> Result<(), String> {
        if !self.current_section_nodes.is_empty() {
            // Flush any remaining trailing content as an 'EOF' section
            let trailing_nodes = std::mem::take(&mut self.current_section_nodes);
            if let Err(e) = self.serialize_section_to_md("End of Document", 1, &trailing_nodes) {
                return Err(format!("IO Error during flush: {}", e));
            }
        }
        Ok(())
    }
}
