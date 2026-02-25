use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A stable identifier generated from content and structure path,
/// NOT physical properties like page numbers, ensuring robust cross-file diffs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StableId(pub u64);

impl StableId {
    /// Generates a deterministic hash for a node based on its path and text content.
    pub fn generate(path_context: &str, content: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        path_context.hash(&mut hasher);
        content.hash(&mut hasher);
        StableId(hasher.finish())
    }
}

/// The logical chunk of a document (e.g., from one H1 to the next).
/// This is the unit of the "Write-through & Release" memory strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub level: u8,
    pub title: String,
    pub nodes: Vec<Node>,
    pub id: StableId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Node {
    Heading {
        level: u8,
        text: String,
        id: StableId,
    },
    Paragraph {
        text: String,
        id: StableId,
    },
    Table(TableDefinition),
    /// Metadata node keeping track of physical bounds if needed (stripped before Markdown gen).
    Fragment {
        page_index: u32,
        id: StableId,
    },
}

/// A Table parsed from the document stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDefinition {
    pub id: StableId,
    pub rows: Vec<Row>,
    /// Flags if this table violates the 'Column Consistency' heuristic.
    pub is_broken: bool,
    /// Enforced column count based on the 80% consensus rule.
    pub expected_columns: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    pub cells: Vec<Cell>,
    pub row_type: RowType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RowType {
    Header,
    Data,
    Total,
    /// A row merged into the previous one due to the 'Row Cohesion' heuristic (e.g. wrapped description text).
    Description,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub raw_text: String,
    /// Extracted via 'Numeric Sanitization'. Only valid f64 values are propagated.
    pub numeric_value: Option<f64>,
}

/// The Lightweight Index for Diffing.
///
/// This is kept in RAM even after the full `Section` AST is flushed to disk via Markdown.
/// It enables rapid R01 (Arithmetic) and R05 (Cross-file) checks using < 100MB RAM even for huge BOQs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumericIndexEntry {
    pub section_id: u64,
    pub table_id: u64,
    pub row_idx: usize,
    pub col_idx: usize,
    pub numeric_value: f64,
}
