//! Calculator: TableTruth → ProjectTruth
//!
//! **Contract:**
//! - Deterministic arithmetic only
//! - No heuristics
//! - No business logic
//! - Fixed rules for status determination
//! - Zero dependencies (No Polars)

use crate::Result;
use iron_table::{
    DeviationSummary, Financials, LineageEntry, ProjectStatus, ProjectTruth,
    SystemMetrics, TableTruth,
};
use std::collections::HashMap;

/// Derive ProjectTruth from a set of validated TableTruths.
///
/// **Rules:**
/// - All calculations are deterministic
/// - Status is computed by fixed thresholds
/// - No AI/ML inference
pub fn derive_project_truth(tables: &[TableTruth], timestamp: String) -> Result<ProjectTruth> {
    let mut financials = Financials {
        total_cost: 0.0,
        total_paid: 0.0,
        remaining: 0.0,
    };

    let mut budget_sum = 0.0;
    let mut actual_sum = 0.0;
    let mut total_rows = 0;
    let mut lineage: HashMap<String, Vec<LineageEntry>> = HashMap::new();

    // Iterate through all tables and aggregate values
    for table in tables {
        total_rows += table.rows.len();

        for row in &table.rows {
            for (col_idx, cell) in row.cells.iter().enumerate() {
                let col_name = table.schema.columns.get(col_idx).map(|c| c.name.as_str()).unwrap_or("");

                match col_name {
                    "total_cost" => {
                        if let Some(val) = cell.value.as_float() {
                            financials.total_cost += val;
                            lineage.entry("total_cost".to_string()).or_default().push(LineageEntry {
                                source_table: table.table_id.clone(),
                                row_idx: row.row_idx,
                                col_idx,
                                contribution: val,
                            });
                        }
                    }
                    "total_paid" => {
                        if let Some(val) = cell.value.as_float() {
                            financials.total_paid += val;
                            lineage.entry("total_paid".to_string()).or_default().push(LineageEntry {
                                source_table: table.table_id.clone(),
                                row_idx: row.row_idx,
                                col_idx,
                                contribution: val,
                            });
                        }
                    }
                    "budget" => {
                        if let Some(val) = cell.value.as_float() {
                            budget_sum += val;
                            lineage.entry("budget".to_string()).or_default().push(LineageEntry {
                                source_table: table.table_id.clone(),
                                row_idx: row.row_idx,
                                col_idx,
                                contribution: val,
                            });
                        }
                    }
                    "actual" => {
                        if let Some(val) = cell.value.as_float() {
                            actual_sum += val;
                            lineage.entry("actual".to_string()).or_default().push(LineageEntry {
                                source_table: table.table_id.clone(),
                                row_idx: row.row_idx,
                                col_idx,
                                contribution: val,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    financials.remaining = financials.total_cost - financials.total_paid;

    let absolute = actual_sum - budget_sum;
    let percentage = if budget_sum != 0.0 {
        (absolute / budget_sum) * 100.0
    } else {
        0.0
    };

    let deviation = DeviationSummary {
        percentage,
        absolute,
    };

    let status = determine_status(&deviation);

    let mut project_truth = ProjectTruth {
        project_name: "Consolidated Dashboard".to_string(),
        last_updated: timestamp,
        data_source: "iron_engine".to_string(),

        project_status: status,
        status_reason: "Deterministic calculation".to_string(),

        financials,
        deviation,
        top_risks: vec![],
        pending_actions: vec![],
        verdicts: vec![],
        metrics: SystemMetrics {
            table_count: tables.len(),
            row_count: total_rows,
            processing_time_ms: 0,
        },
        lineage,
    };

    // Run the ValidationEngine against the calculated truth
    let context = crate::numeric_validator::ValidationContext {
        project_truth: &project_truth,
        raw_tables: tables,
        comparison_pairs: &[],
    };

    project_truth.verdicts = crate::numeric_validator::ValidationEngine::verify(context);

    Ok(project_truth)
}

fn determine_status(deviation: &DeviationSummary) -> ProjectStatus {
    if deviation.percentage < 5.0 {
        ProjectStatus::Safe
    } else if deviation.percentage < 15.0 {
        ProjectStatus::Warning
    } else {
        ProjectStatus::Critical
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iron_table::{DataType, ColumnDef, TableSchema, TableRow, TableCell, BoundingBox, CellValue};
    use std::path::PathBuf;

    #[test]
    fn test_calculator_pure_rust_logic() {
        let table = TableTruth {
            table_id: "test".to_string(),
            source_file: PathBuf::from("test.pdf"),
            source_page: 1,
            schema: TableSchema {
                columns: vec![
                    ColumnDef { name: "total_cost".to_string(), dtype: DataType::Float64, unit: None, nullable: false, is_critical: true },
                    ColumnDef { name: "total_paid".to_string(), dtype: DataType::Float64, unit: None, nullable: false, is_critical: true },
                ],
                row_count: 1,
                col_count: 2,
            },
            rows: vec![
                TableRow {
                    row_idx: 0,
                    cells: vec![
                        TableCell { global_id: "1".into(), row_idx: 0, col_idx: 0, value: CellValue::Float(100.0), bbox: dummy_bbox(), confidence: 1.0, source_text: "".into(), encoding_status: Default::default(), encoding_evidence: None },
                        TableCell { global_id: "2".into(), row_idx: 0, col_idx: 1, value: CellValue::Float(40.0), bbox: dummy_bbox(), confidence: 1.0, source_text: "".into(), encoding_status: Default::default(), encoding_evidence: None },
                    ]
                }
            ],
            extraction_meta: iron_table::ExtractionMeta { tool_version: "".into(), timestamp: "".into(), confidence_score: 1.0 },
            bbox: dummy_bbox(),
        };

        let truth = derive_project_truth(&[table], "now".into()).unwrap();
        assert_eq!(truth.financials.total_cost, 100.0);
        assert_eq!(truth.financials.total_paid, 40.0);
        assert_eq!(truth.financials.remaining, 60.0);
    }

    fn dummy_bbox() -> BoundingBox {
        BoundingBox { x: 0.0, y: 0.0, width: 0.0, height: 0.0, page: 1 }
    }
}
