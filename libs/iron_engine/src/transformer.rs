//! Transformer: TableTruth → Polars DataFrame
//!
//! **Contract:**
//! - Explicit Series construction only
//! - 1:1 column mapping
//! - No inference, no coercion
//! - Reject on schema mismatch

use crate::{EngineError, Result};
use iron_table::{ColumnDef, DataType as IronDataType, TableTruth};
use polars::prelude::*;

/// Convert a validated TableTruth into a Polars DataFrame.
///
/// **Rules:**
/// - Schema is explicit (from TableTruth.schema)
/// - No type inference
/// - No silent null filling
/// - Errors on any ambiguity
pub fn to_dataframe(table: &TableTruth) -> Result<DataFrame> {
    // Validate that all rows have correct cell count (Iron Truth Contract enforcement)
    for row in &table.rows {
        if row.cells.len() != table.schema.col_count {
            return Err(EngineError::SchemaMismatch(format!(
                "Row {} has {} cells but schema expects {} columns",
                row.row_idx,
                row.cells.len(),
                table.schema.col_count
            )));
        }
    }

    let mut series_vec: Vec<Series> = Vec::new();

    for col_def in &table.schema.columns {
        // 1. Value Series
        let series = build_series(table, col_def)?;
        series_vec.push(series);

        // 2. Lineage Series (Shadow)
        let lineage_series = build_lineage_series(table, col_def)?;
        series_vec.push(lineage_series);
    }

    // Convert Series to Column for DataFrame
    let columns: Vec<Column> = series_vec.into_iter().map(Column::from).collect();
    DataFrame::new(columns).map_err(|e| e.into())
}

fn build_lineage_series(table: &TableTruth, col_def: &ColumnDef) -> Result<Series> {
    let col_idx = table
        .schema
        .columns
        .iter()
        .position(|c| c.name == col_def.name)
        .ok_or_else(|| {
            EngineError::SchemaMismatch(format!("Column '{}' not found in schema", col_def.name))
        })?;

    let values: Vec<Option<String>> = table
        .rows
        .iter()
        .map(|row| row.cells.get(col_idx).map(|cell| cell.global_id.clone()))
        .collect();

    Ok(Series::new(
        format!("_lineage_{}", col_def.name).into(),
        &values,
    ))
}

/// Build a single Series from a column definition.
///
/// **Deterministic mapping:**
/// - Int → Int64Series
/// - Float64 → Float64Series
/// - Utf8 → Utf8Series
/// - Date → DateSeries
fn build_series(table: &TableTruth, col_def: &ColumnDef) -> Result<Series> {
    let col_idx = table
        .schema
        .columns
        .iter()
        .position(|c| c.name == col_def.name)
        .ok_or_else(|| {
            EngineError::SchemaMismatch(format!("Column '{}' not found in schema", col_def.name))
        })?;

    match &col_def.dtype {
        IronDataType::Int => {
            let values: Vec<Option<i64>> = table
                .rows
                .iter()
                .map(|row| row.cells.get(col_idx).and_then(|cell| cell.value.as_int()))
                .collect();
            Ok(Series::new((&col_def.name).into(), &values))
        }
        IronDataType::Float64 => {
            let values: Vec<Option<f64>> = table
                .rows
                .iter()
                .map(|row| {
                    row.cells
                        .get(col_idx)
                        .and_then(|cell| cell.value.as_float())
                })
                .collect();
            Ok(Series::new((&col_def.name).into(), &values))
        }
        IronDataType::Utf8 => {
            let values: Vec<Option<String>> = table
                .rows
                .iter()
                .map(|row| {
                    row.cells
                        .get(col_idx)
                        .and_then(|cell| cell.value.as_str().map(|s| s.to_string()))
                })
                .collect();
            Ok(Series::new((&col_def.name).into(), &values))
        }
        IronDataType::Date => {
            // TODO: Implement date parsing with deterministic rules
            Err(EngineError::InvalidData(
                "Date type not yet implemented".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iron_table::{
        BoundingBox, CellValue, ColumnDef, DataType, EncodingStatus, ExtractionMeta, TableCell,
        TableRow, TableSchema, TableTruth,
    };
    use std::path::PathBuf;

    /// Helper: Create a minimal valid TableTruth for testing
    fn create_test_table_truth() -> TableTruth {
        TableTruth {
            table_id: "test-table-001".to_string(),
            source_file: PathBuf::from("test.pdf"),
            source_page: 1,
            schema: TableSchema {
                columns: vec![
                    ColumnDef {
                        name: "id".to_string(),
                        dtype: DataType::Int,
                        unit: None,
                        nullable: false,
                        is_critical: true,
                    },
                    ColumnDef {
                        name: "amount".to_string(),
                        dtype: DataType::Float64,
                        unit: Some("VND".to_string()),
                        nullable: false,
                        is_critical: true,
                    },
                    ColumnDef {
                        name: "description".to_string(),
                        dtype: DataType::Utf8,
                        unit: None,
                        nullable: true,
                        is_critical: false,
                    },
                ],
                row_count: 2,
                col_count: 3,
            },
            rows: vec![
                TableRow {
                    row_idx: 0,
                    cells: vec![
                        TableCell {
                            global_id: "test-table-001_0_0".to_string(),
                            row_idx: 0,
                            col_idx: 0,
                            value: CellValue::Int(1),
                            bbox: BoundingBox {
                                x: 0.0,
                                y: 0.0,
                                width: 10.0,
                                height: 10.0,
                                page: 1,
                            },
                            confidence: 0.95,
                            source_text: "1".to_string(),
                            encoding_status: EncodingStatus::Clean,
                            encoding_evidence: None,
                        },
                        TableCell {
                            global_id: "test-table-001_0_1".to_string(),
                            row_idx: 0,
                            col_idx: 1,
                            value: CellValue::Float(1000.50),
                            bbox: BoundingBox {
                                x: 10.0,
                                y: 0.0,
                                width: 10.0,
                                height: 10.0,
                                page: 1,
                            },
                            confidence: 0.92,
                            source_text: "1000.50".to_string(),
                            encoding_status: EncodingStatus::Clean,
                            encoding_evidence: None,
                        },
                        TableCell {
                            global_id: "test-table-001_0_2".to_string(),
                            row_idx: 0,
                            col_idx: 2,
                            value: CellValue::Text("Item A".to_string()),
                            bbox: BoundingBox {
                                x: 20.0,
                                y: 0.0,
                                width: 10.0,
                                height: 10.0,
                                page: 1,
                            },
                            confidence: 0.88,
                            source_text: "Item A".to_string(),
                            encoding_status: EncodingStatus::Clean,
                            encoding_evidence: None,
                        },
                    ],
                },
                TableRow {
                    row_idx: 1,
                    cells: vec![
                        TableCell {
                            global_id: "test-table-001_1_0".to_string(),
                            row_idx: 1,
                            col_idx: 0,
                            value: CellValue::Int(2),
                            bbox: BoundingBox {
                                x: 0.0,
                                y: 10.0,
                                width: 10.0,
                                height: 10.0,
                                page: 1,
                            },
                            confidence: 0.96,
                            source_text: "2".to_string(),
                            encoding_status: EncodingStatus::Clean,
                            encoding_evidence: None,
                        },
                        TableCell {
                            global_id: "test-table-001_1_1".to_string(),
                            row_idx: 1,
                            col_idx: 1,
                            value: CellValue::Float(2500.75),
                            bbox: BoundingBox {
                                x: 10.0,
                                y: 10.0,
                                width: 10.0,
                                height: 10.0,
                                page: 1,
                            },
                            confidence: 0.94,
                            source_text: "2500.75".to_string(),
                            encoding_status: EncodingStatus::Clean,
                            encoding_evidence: None,
                        },
                        TableCell {
                            global_id: "test-table-001_1_2".to_string(),
                            row_idx: 1,
                            col_idx: 2,
                            value: CellValue::Text("Item B".to_string()),
                            bbox: BoundingBox {
                                x: 20.0,
                                y: 10.0,
                                width: 10.0,
                                height: 10.0,
                                page: 1,
                            },
                            confidence: 0.90,
                            source_text: "Item B".to_string(),
                            encoding_status: EncodingStatus::Clean,
                            encoding_evidence: None,
                        },
                    ],
                },
            ],
            extraction_meta: ExtractionMeta {
                tool_version: "docling-v2".to_string(),
                timestamp: "2026-01-31T13:00:00Z".to_string(),
                confidence_score: 0.92,
            },
            bbox: BoundingBox {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 50.0,
                page: 1,
            },
        }
    }

    /// LAYER 1 TEST 1: Exact Schema Match
    /// Verifies that DataFrame has correct column names, types, and row count
    #[test]
    fn test_to_dataframe_exact_schema_match() {
        let table = create_test_table_truth();
        let df = to_dataframe(&table).expect("DataFrame conversion should succeed");

        // Verify column count (3 data columns + 3 lineage columns)
        assert_eq!(df.width(), 6, "DataFrame should have 6 columns");

        // Verify row count
        assert_eq!(df.height(), 2, "DataFrame should have 2 rows");

        // Verify column names
        let col_names: Vec<String> = df
            .get_column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(col_names.len(), 6);
        assert!(col_names.contains(&"id".to_string()));
        assert!(col_names.contains(&"_lineage_id".to_string()));
        assert!(col_names.contains(&"amount".to_string()));
        assert!(col_names.contains(&"_lineage_amount".to_string()));
        assert!(col_names.contains(&"description".to_string()));
        assert!(col_names.contains(&"_lineage_description".to_string()));

        // Verify column types (using Polars DataType)
        assert!(matches!(
            df.column("id").unwrap().dtype(),
            polars::prelude::DataType::Int64
        ));
        assert!(matches!(
            df.column("amount").unwrap().dtype(),
            polars::prelude::DataType::Float64
        ));
        assert!(matches!(
            df.column("description").unwrap().dtype(),
            polars::prelude::DataType::String
        ));

        // Verify actual values
        let id_col = df.column("id").unwrap();
        let id_series = id_col.i64().unwrap();
        assert_eq!(id_series.get(0), Some(1));
        assert_eq!(id_series.get(1), Some(2));

        let amount_col = df.column("amount").unwrap();
        let amount_series = amount_col.f64().unwrap();
        assert_eq!(amount_series.get(0), Some(1000.50));
        assert_eq!(amount_series.get(1), Some(2500.75));
    }

    /// LAYER 1 TEST 2: Schema Mismatch Rejection
    /// Verifies that mismatched cell counts cause proper errors
    #[test]
    fn test_to_dataframe_reject_schema_mismatch() {
        let mut table = create_test_table_truth();

        // Corrupt the table: remove one cell from first row
        table.rows[0].cells.pop();

        let result = to_dataframe(&table);

        // Should fail because row has 2 cells but schema expects 3
        assert!(
            result.is_err(),
            "Should reject table with mismatched cell count"
        );
    }

    /// LAYER 1 TEST 3: Deterministic Conversion
    /// Verifies that same input produces identical output
    #[test]
    fn test_dataframe_deterministic() {
        let table = create_test_table_truth();

        let df1 = to_dataframe(&table).expect("First conversion should succeed");
        let df2 = to_dataframe(&table).expect("Second conversion should succeed");

        // Verify schema equality
        assert_eq!(
            df1.get_column_names(),
            df2.get_column_names(),
            "Column names must be identical"
        );
        assert_eq!(df1.dtypes(), df2.dtypes(), "Data types must be identical");
        assert_eq!(df1.height(), df2.height(), "Row counts must be identical");

        // Verify value equality for each column
        for col_name in df1.get_column_names() {
            let col1 = df1.column(col_name).unwrap();
            let col2 = df2.column(col_name).unwrap();

            assert_eq!(col1, col2, "Column '{}' values must be identical", col_name);
        }
    }
}
