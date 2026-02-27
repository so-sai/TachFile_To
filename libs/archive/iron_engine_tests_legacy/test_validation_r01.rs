use iron_engine::{
    validation_engine::{ValidationContext, ValidationEngine},
    DeviationSummary, Financials, ProjectStatus, ProjectTruth, SystemMetrics, ViolationType,
};
use std::collections::HashMap;

/// Helper: Create a Mock ProjectTruth with accurate or corrupted financials
fn mock_project_truth(cost: f64, paid: f64, remaining: f64) -> ProjectTruth {
    ProjectTruth {
        project_name: "Test R01".to_string(),
        last_updated: "2026-01-01".to_string(),
        data_source: "Test".to_string(),
        project_status: ProjectStatus::Safe,
        status_reason: "Test".to_string(),
        financials: Financials {
            total_cost: cost,
            total_paid: paid,
            remaining: remaining,
        },
        deviation: DeviationSummary {
            percentage: 0.0,
            absolute: 0.0,
        },
        top_risks: vec![],
        pending_actions: vec![],
        metrics: SystemMetrics {
            table_count: 0,
            row_count: 0,
            processing_time_ms: 0,
        },
        lineage: HashMap::new(),
        verdicts: vec![],
    }
}

#[test]
fn test_r01_exact_match() {
    // 1000 - 300 = 700. Exact.
    let truth = mock_project_truth(1000.0, 300.0, 700.0);
    let context = ValidationContext {
        project_truth: &truth,
        raw_tables: &[],
        comparison_pairs: &[],
    };

    let verdicts = ValidationEngine::verify(context);

    // Should have 0 violations
    assert!(verdicts.is_empty(), "Exact match should have 0 violations");
}

#[test]
fn test_r01_rounding_tolerance_clean() {
    // 1000 - 300 = 700.
    // Reported: 700.0000000001 (Floating point ghost)
    // Diff: 0.0000000001 < 1.0 Epsilon

    let truth = mock_project_truth(1000.0, 300.0, 700.0000000001);
    let context = ValidationContext {
        project_truth: &truth,
        raw_tables: &[],
        comparison_pairs: &[],
    };

    let verdicts = ValidationEngine::verify(context);

    assert!(
        verdicts.is_empty(),
        "Rounding error < Epsilon should be CLEAN"
    );
}

#[test]
fn test_r01_rounding_tolerance_clean_large_epsilon() {
    // 1000 - 300 = 700.
    // Reported: 700.9 (Less than 1.0 diff)
    // Diff: 0.9 < 1.0 Epsilon

    let truth = mock_project_truth(1000.0, 300.0, 700.9);
    let context = ValidationContext {
        project_truth: &truth,
        raw_tables: &[],
        comparison_pairs: &[],
    };

    let verdicts = ValidationEngine::verify(context);

    assert!(
        verdicts.is_empty(),
        "Diff 0.9 should be CLEAN under Epsilon 1.0"
    );
}

#[test]
fn test_r01_critical_mismatch() {
    // 1000 - 300 = 700.
    // Reported: 650.0 (Manual tampering)
    // Diff: 50.0 > 1.0 Epsilon

    let truth = mock_project_truth(1000.0, 300.0, 650.0);
    let context = ValidationContext {
        project_truth: &truth,
        raw_tables: &[],
        comparison_pairs: &[],
    };

    let verdicts = ValidationEngine::verify(context);

    assert_eq!(verdicts.len(), 1, "Must catch 1 critical violation");

    let v = &verdicts[0];
    assert!(
        matches!(v.violation, ViolationType::MathError),
        "Violation must be MathError"
    );
    assert_eq!(v.severity, 3, "Severity must be Critical (3)");
    assert!(
        v.message.contains("Calculated Remaining (700)"),
        "Message must specify calculated truth"
    );
}
