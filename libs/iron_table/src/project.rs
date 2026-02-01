use crate::contract::{TableTruth, CellValue};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::collections::HashMap;

// ============================================================================
// CORE IDENTIFIERS
// ============================================================================

pub type TableId = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TableRef(pub TableId);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ColumnRef {
    pub table_id: TableId,
    pub col_idx: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CellRef {
    pub table_id: TableId,
    pub row_idx: usize,
    pub col_idx: usize,
}

// ============================================================================
// CONSISTENCY RULES (DECLARATIVE)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "rule_type")]
pub enum ConsistencyRule {
    /// Ensures two cells must have identical values (e.g., Project Name).
    ExactMatch {
        source: CellRef,
        target: CellRef,
    },
    /// Ensures sum of a column equals a target cell (e.g., Sum(Details) == Total).
    SumMatch {
        source_table: TableRef,
        source_column: ColumnRef,
        target: CellRef,
    },
}

// ============================================================================
// PROJECT GRAPH & RELATIONS (MISSION 030)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RelationKind {
    /// The primary document (e.g., Main Contract)
    ParentOf,
    /// An attachment or supplement (e.g., Appendix, Addendum)
    AppendixTo,
    /// Cross-reference for validation (e.g., BoQ item references standard price list)
    VerifiedBy,
    /// Informational link
    Reference,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Relation {
    pub from: TableId,
    pub to: TableId,
    pub kind: RelationKind,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectGraph {
    pub project_id: String,
    pub tables: HashMap<TableId, TableTruth>,
    pub rules: Vec<ConsistencyRule>,
    pub relations: Vec<Relation>,
}

#[derive(Error, Debug)]
pub enum ProjectRejection {
    #[error("Missing Table: {0}")]
    MissingTable(String),

    #[error("Missing Reference: {0}")]
    MissingReference(String),

    #[error("Consistency Violation: {0}")]
    ConsistencyViolation(String),
    
    #[error("Type Mismatch: {0}")]
    TypeMismatch(String),

    #[error("Circular Dependency: {0}")]
    CircularDependency(String),
}

impl ProjectGraph {
    pub fn new(id: String) -> Self {
        Self {
            project_id: id,
            tables: HashMap::new(),
            rules: Vec::new(),
            relations: Vec::new(),
        }
    }

    pub fn add_table(&mut self, table: TableTruth) {
        self.tables.insert(table.table_id.clone(), table);
    }

    pub fn add_rule(&mut self, rule: ConsistencyRule) {
        self.rules.push(rule);
    }

    pub fn add_relation(&mut self, from: TableId, to: TableId, kind: RelationKind) {
        self.relations.push(Relation { from, to, kind, metadata: None });
    }

    /// Validates the project graph for internal consistency and relational integrity.
    pub fn validate_project(&self) -> Result<(), ProjectRejection> {
        // 1. Consistency Rules (The Supreme Court)
        for rule in &self.rules {
            match rule {
                ConsistencyRule::ExactMatch { source, target } => {
                    self.validate_exact_match(source, target)?;
                }
                ConsistencyRule::SumMatch { source_table, source_column, target } => {
                    self.validate_sum_match(source_table, source_column, target)?;
                }
            }
        }
        
        // 2. Relational Integrity
        for rel in &self.relations {
            if !self.tables.contains_key(&rel.from) {
                return Err(ProjectRejection::MissingTable(rel.from.clone()));
            }
            if !self.tables.contains_key(&rel.to) {
                return Err(ProjectRejection::MissingTable(rel.to.clone()));
            }
        }
        
        Ok(())
    }

    fn validate_exact_match(&self, source: &CellRef, target: &CellRef) -> Result<(), ProjectRejection> {
        let val_source = self.get_cell_value(source)?;
        let val_target = self.get_cell_value(target)?;

        if val_source != val_target {
            return Err(ProjectRejection::ConsistencyViolation(format!(
                "ExactMatch Failed: {:?} != {:?}",
                val_source, val_target
            )));
        }
        Ok(())
    }

    fn validate_sum_match(
        &self,
        source_table_ref: &TableRef,
        source_col_ref: &ColumnRef,
        target: &CellRef
    ) -> Result<(), ProjectRejection> {
        let table = self.tables.get(&source_table_ref.0)
            .ok_or_else(|| ProjectRejection::MissingTable(source_table_ref.0.clone()))?;

        // 1. Get Target Value
        let val_target = self.get_cell_value(target)?;
        
        // 2. Calculate Sum of Source Column (Strictly for validation comparison)
        // We accumulate into a float or int depending on target type
        // For simplicity and strictness in QS, we assume Float64 mostly, but handle Int.
        
        let target_float = match val_target {
            CellValue::Int(i) => *i as f64,
            CellValue::Float(f) => *f,
            _ => return Err(ProjectRejection::TypeMismatch("Target must be numeric for SumMatch".into())),
        };

        let mut sum: f64 = 0.0;
        
        for row in &table.rows {
             if let Some(cell) = row.cells.iter().find(|c| c.col_idx == source_col_ref.col_idx) {
                 match cell.value {
                     CellValue::Int(i) => sum += i as f64,
                     CellValue::Float(f) => sum += f,
                     CellValue::Null => {}, // Treat null as 0
                     _ => {
                         // Skip header row gracefully if it contains text
                         if row.row_idx == 0 { continue; }
                         return Err(ProjectRejection::TypeMismatch(format!("Column contains non-numeric data at row {}", row.row_idx)));
                     }
                 }
             }
        }

        // 3. Compare (Strict Equality, NO Epsilon)
        if sum != target_float {
             return Err(ProjectRejection::ConsistencyViolation(format!(
                "SumMatch Failed: Source Sum {} != Target Value {}",
                sum, target_float
            )));
        }

        Ok(())
    }

    fn get_cell_value(&self, cell_ref: &CellRef) -> Result<&CellValue, ProjectRejection> {
        let table = self.tables.get(&cell_ref.table_id)
            .ok_or_else(|| ProjectRejection::MissingTable(cell_ref.table_id.clone()))?;
        
        let row = table.rows.iter().find(|r| r.row_idx == cell_ref.row_idx)
             .ok_or_else(|| ProjectRejection::MissingReference(format!("Row {} not found in table {}", cell_ref.row_idx, cell_ref.table_id)))?;

        let cell = row.cells.iter().find(|c| c.col_idx == cell_ref.col_idx)
             .ok_or_else(|| ProjectRejection::MissingReference(format!("Col {} not found in row {} table {}", cell_ref.col_idx, cell_ref.row_idx, cell_ref.table_id)))?;

        Ok(&cell.value)
    }
}

// ============================================================================
// TESTS
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{TableTruth, TableRow, TableCell, CellValue, BoundingBox, TableSchema, ExtractionMeta, EncodingStatus};
    use std::path::PathBuf;

    fn create_mock_table(id: &str, data: Vec<Vec<CellValue>>) -> TableTruth {
        let rows: Vec<TableRow> = data.into_iter().enumerate().map(|(row_idx, row_data)| {
            TableRow {
                row_idx,
                cells: row_data.into_iter().enumerate().map(|(col_idx, val)| {
                    TableCell {
                        global_id: format!("{}_{}_{}", id, row_idx, col_idx),
                        row_idx,
                        col_idx,
                        value: val,
                        bbox: BoundingBox { x:0.0, y:0.0, width:0.0, height:0.0, page:1 },
                        confidence: 1.0,
                        source_text: "".into(),
                        encoding_status: EncodingStatus::Clean,
                        encoding_evidence: None,
                    }
                }).collect()
            }
        }).collect();

        TableTruth {
            table_id: id.into(),
            source_file: PathBuf::from("mock.pdf"),
            source_page: 1,
            schema: TableSchema { columns: vec![], row_count: rows.len(), col_count: 0 },
            rows,
            extraction_meta: ExtractionMeta { tool_version: "".into(), timestamp: "".into(), confidence_score: 1.0 },
            bbox: BoundingBox { x:0.0, y:0.0, width:0.0, height:0.0, page:1 },
        }
    }

    #[test]
    fn test_exact_match_pass() {
        let mut project = ProjectGraph::new("test".into());
        let t1 = create_mock_table("t1", vec![vec![CellValue::Text("A".into())]]);
        let t2 = create_mock_table("t2", vec![vec![CellValue::Text("A".into())]]);
        
        project.add_table(t1);
        project.add_table(t2);

        project.add_rule(ConsistencyRule::ExactMatch {
            source: CellRef { table_id: "t1".into(), row_idx: 0, col_idx: 0 },
            target: CellRef { table_id: "t2".into(), row_idx: 0, col_idx: 0 },
        });

        assert!(project.validate_project().is_ok());
    }

    #[test]
    fn test_exact_match_fail() {
        let mut project = ProjectGraph::new("test".into());
        let t1 = create_mock_table("t1", vec![vec![CellValue::Text("A".into())]]);
        let t2 = create_mock_table("t2", vec![vec![CellValue::Text("B".into())]]);
        
        project.add_table(t1);
        project.add_table(t2);

        project.add_rule(ConsistencyRule::ExactMatch {
            source: CellRef { table_id: "t1".into(), row_idx: 0, col_idx: 0 },
            target: CellRef { table_id: "t2".into(), row_idx: 0, col_idx: 0 },
        });

        match project.validate_project() {
            Err(ProjectRejection::ConsistencyViolation(_)) => {},
            _ => panic!("Should fail"),
        }
    }

    #[test]
    fn test_sum_match_pass() {
        let mut project = ProjectGraph::new("test".into());
        // Source table: Column 0 has 40.0 and 60.0
        let t1 = create_mock_table("t1", vec![
            vec![CellValue::Float(40.0)],
            vec![CellValue::Float(60.0)],
        ]);
        // Target table: Cell (0,0) has 100.0
        let t2 = create_mock_table("t2", vec![
             vec![CellValue::Float(100.0)]
        ]);
        
        project.add_table(t1);
        project.add_table(t2);

        project.add_rule(ConsistencyRule::SumMatch {
            source_table: TableRef("t1".into()),
            source_column: ColumnRef { table_id: "t1".into(), col_idx: 0 },
            target: CellRef { table_id: "t2".into(), row_idx: 0, col_idx: 0 },
        });

        assert!(project.validate_project().is_ok());
    }

    #[test]
    fn test_sum_match_fail_strict() {
        let mut project = ProjectGraph::new("test".into());
        // Source table: 40.0 + 59.0 = 99.0
        let t1 = create_mock_table("t1", vec![
            vec![CellValue::Float(40.0)],
            vec![CellValue::Float(59.0)],
        ]);
        // Target table: 100.0
        let t2 = create_mock_table("t2", vec![
             vec![CellValue::Float(100.0)]
        ]);
        
        project.add_table(t1);
        project.add_table(t2);

        project.add_rule(ConsistencyRule::SumMatch {
            source_table: TableRef("t1".into()),
            source_column: ColumnRef { table_id: "t1".into(), col_idx: 0 },
            target: CellRef { table_id: "t2".into(), row_idx: 0, col_idx: 0 },
        });

        match project.validate_project() {
            Err(ProjectRejection::ConsistencyViolation(msg)) => {
                println!("{}", msg); // Expected: 99 != 100
                assert!(msg.contains("99") && msg.contains("100"));
            },
            _ => panic!("Should fail with ConsistencyViolation"),
        }
    }
}
