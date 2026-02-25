use iron_engine::{
    validation_engine::{ValidationContext, ValidationEngine},
    DeviationSummary, Financials, ProjectStatus, ProjectTruth, SystemMetrics, TableTruth,
    ViolationType,
};
use iron_table::{
    BoundingBox, CellValue, ColumnDef, DataType, EncodingStatus, ExtractionMeta, TableCell,
    TableRow, TableSchema,
};
use std::collections::HashMap;

/// Helper: Create Mock ProjectTruth (Dummy)
fn mock_project_truth() -> ProjectTruth {
    ProjectTruth {
        project_name: "Test R05".to_string(),
        last_updated: "2026-01-01".to_string(),
        data_source: "Test".to_string(),
        project_status: ProjectStatus::Safe,
        status_reason: "Test".to_string(),
        financials: Financials {
            total_cost: 0.0,
            total_paid: 0.0,
            remaining: 0.0,
        },
        deviation: DeviationSummary {
            percentage: 0.0,
            absolute: 0.0,
        },
        top_risks: vec![],
        pending_actions: vec![],
        verdicts: vec![],
        metrics: SystemMetrics {
            table_count: 0,
            row_count: 0,
            processing_time_ms: 0,
        },
        lineage: HashMap::new(),
    }
}

/// Helper: Create Mock TableTruth with a single numeric cell
fn mock_table(id: &str, row_val: f64) -> TableTruth {
    let bbox = BoundingBox {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        page: 1,
    };

    let cell = TableCell {
        global_id: format!("{}_0_0", id),
        row_idx: 0,
        col_idx: 0,
        value: CellValue::Float(row_val),
        bbox: bbox.clone(),
        confidence: 1.0,
        source_text: row_val.to_string(),
        encoding_status: EncodingStatus::Clean,
        encoding_evidence: None,
    };

    let row = TableRow {
        row_idx: 0,
        cells: vec![cell],
    };

    TableTruth {
        table_id: id.to_string(),
        source_file: std::path::PathBuf::from("mock.pdf"),
        source_page: 1,
        schema: TableSchema {
            columns: vec![ColumnDef {
                name: "Value".to_string(),
                dtype: DataType::Float64,
                unit: None,
                nullable: false,
                is_critical: false,
            }],
            row_count: 1,
            col_count: 1,
        },
        rows: vec![row],
        extraction_meta: ExtractionMeta {
            tool_version: "Test".to_string(),
            timestamp: "".to_string(),
            confidence_score: 1.0,
        },
        bbox,
    }
}

#[test]
fn test_r05_exact_match() {
    let truth = mock_project_truth();
    let t1 = mock_table("PDF", 100.0);
    let t2 = mock_table("XLS", 100.0);

    let pair = (&t1, &t2);

    let context = ValidationContext {
        project_truth: &truth,
        raw_tables: &[],
        comparison_pairs: &[pair],
    };

    let verdicts = ValidationEngine::verify(context);

    // Filter for Mismatch only (R05)
    let mismatches: Vec<_> = verdicts
        .iter()
        .filter(|v| matches!(v.violation, ViolationType::Mismatch))
        .collect();

    assert!(
        mismatches.is_empty(),
        "Exact match should have 0 mismatches"
    );
}

#[test]
fn test_r05_tolerance_match() {
    let truth = mock_project_truth();
    let t1 = mock_table("PDF", 100.0);
    let t2 = mock_table("XLS", 100.9); // Diff 0.9 < 1.0 Epsilon

    let pair = (&t1, &t2);

    let context = ValidationContext {
        project_truth: &truth,
        raw_tables: &[],
        comparison_pairs: &[pair],
    };

    let verdicts = ValidationEngine::verify(context);

    let mismatches: Vec<_> = verdicts
        .iter()
        .filter(|v| matches!(v.violation, ViolationType::Mismatch))
        .collect();

    assert!(
        mismatches.is_empty(),
        "Diff 0.9 should be ignored (Epsilon Law)"
    );
}

#[test]
fn test_r05_mismatch_detected() {
    let truth = mock_project_truth();
    let t1 = mock_table("PDF", 100.0);
    let t2 = mock_table("XLS", 102.0); // Diff 2.0 > 1.0 Epsilon

    let pair = (&t1, &t2);

    let context = ValidationContext {
        project_truth: &truth,
        raw_tables: &[],
        comparison_pairs: &[pair],
    };

    let verdicts = ValidationEngine::verify(context);

    let mismatches: Vec<_> = verdicts
        .iter()
        .filter(|v| matches!(v.violation, ViolationType::Mismatch))
        .collect();

    assert_eq!(mismatches.len(), 1, "Should detect 1 mismatch");
    assert_eq!(mismatches[0].severity, 3, "Mismatch is Critical");
    assert!(
        mismatches[0].message.contains("Diff: 2"),
        "Message should state diff"
    );
}
