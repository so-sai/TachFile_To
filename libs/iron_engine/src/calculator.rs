//! Calculator: DataFrame → ProjectTruth
//! 
//! **Contract:**
//! - Deterministic arithmetic only
//! - No heuristics
//! - No business logic
//! - Fixed rules for status determination

use polars::prelude::*;
use iron_table::{
    ProjectTruth, ProjectStatus, Financials, DeviationSummary, SystemMetrics
};
use crate::{Result, EngineError};

/// Derive ProjectTruth from a validated DataFrame.
/// 
/// **Rules:**
/// - All calculations are deterministic
/// - Status is computed by fixed thresholds
/// - No AI/ML inference
/// 
/// **Implementation Status:**
/// - Financial aggregation: ✅ Implemented (exact column sums)
/// - Deviation calculation: ✅ Implemented (budget vs actual)
/// - Status determination: ✅ Implemented (hard thresholds: 5%, 15%)
/// 
/// **Determinism:** Timestamp must be provided by caller to ensure idempotence.
pub fn derive_project_truth(df: &DataFrame, timestamp: String) -> Result<ProjectTruth> {
    
    use std::collections::HashMap;

    let financials = calculate_financials(df)?;
    let deviation = calculate_deviation(df)?;
    let status = determine_status(&deviation);
    
    let mut lineage = HashMap::new();
    
    // Collect lineage for key financial metrics
    if df.column("total_cost").is_ok() {
        lineage.insert("total_cost".to_string(), collect_lineage(df, "total_cost"));
    }
    if df.column("total_paid").is_ok() {
        lineage.insert("total_paid".to_string(), collect_lineage(df, "total_paid"));
    }
    if df.column("budget").is_ok() {
        lineage.insert("budget".to_string(), collect_lineage(df, "budget"));
    }
    if df.column("actual").is_ok() {
        lineage.insert("actual".to_string(), collect_lineage(df, "actual"));
    }

    Ok(ProjectTruth {
        project_name: "Consolidated Dashboard".to_string(),
        last_updated: timestamp,
        data_source: "iron_engine".to_string(),
        
        project_status: status,
        status_reason: "Deterministic calculation".to_string(),
        
        financials,
        deviation,
        top_risks: vec![],
        pending_actions: vec![],
        metrics: SystemMetrics {
            table_count: 1,
            row_count: df.height(),
            processing_time_ms: 0,
        },
        lineage,
    })
}

fn collect_lineage(df: &DataFrame, value_col: &str) -> Vec<iron_table::LineageEntry> {
    use iron_table::LineageEntry;
    let mut entries = Vec::new();
    
    let val_column = match df.column(value_col) {
        Ok(c) => c,
        Err(_) => return entries,
    };
    
    let lin_col_name = format!("_lineage_{}", value_col);
    let lin_column = match df.column(&lin_col_name) {
        Ok(c) => c,
        Err(_) => return entries,
    };

    let f64_vals = match val_column.f64() {
        Ok(v) => v,
        Err(_) => return entries,
    };
    
    let lin_vals = match lin_column.str() {
        Ok(l) => l,
        Err(_) => return entries,
    };

    for i in 0..df.height() {
        if let (Some(val), Some(lin_id)) = (f64_vals.get(i), lin_vals.get(i)) {
            if val != 0.0 {
                let parts: Vec<&str> = lin_id.split('_').collect();
                if parts.len() >= 3 {
                    entries.push(LineageEntry {
                        source_table: parts[0..parts.len()-2].join("_"),
                        row_idx: parts[parts.len()-2].parse().unwrap_or(0),
                        col_idx: parts[parts.len()-1].parse().unwrap_or(0),
                        contribution: val,
                    });
                }
            }
        }
    }
    
    entries
}

fn calculate_financials(df: &DataFrame) -> Result<Financials> {
    // Extract and sum financial columns
    let total_cost = if let Ok(col) = df.column("total_cost") {
        col.f64()
            .map_err(|e| EngineError::InvalidData(format!("total_cost column type error: {}", e)))?
            .sum()
            .unwrap_or(0.0)
    } else {
        0.0
    };

    let total_paid = if let Ok(col) = df.column("total_paid") {
        col.f64()
            .map_err(|e| EngineError::InvalidData(format!("total_paid column type error: {}", e)))?
            .sum()
            .unwrap_or(0.0)
    } else {
        0.0
    };

    let remaining = total_cost - total_paid;

    Ok(Financials {
        total_cost,
        total_paid,
        remaining,
    })
}

fn calculate_deviation(df: &DataFrame) -> Result<DeviationSummary> {
    // Extract budget and actual columns
    let budget_sum = if let Ok(col) = df.column("budget") {
        col.f64()
            .map_err(|e| EngineError::InvalidData(format!("budget column type error: {}", e)))?
            .sum()
            .unwrap_or(0.0)
    } else {
        return Ok(DeviationSummary { percentage: 0.0, absolute: 0.0 });
    };

    let actual_sum = if let Ok(col) = df.column("actual") {
        col.f64()
            .map_err(|e| EngineError::InvalidData(format!("actual column type error: {}", e)))?
            .sum()
            .unwrap_or(0.0)
    } else {
        return Ok(DeviationSummary { percentage: 0.0, absolute: 0.0 });
    };

    let absolute = actual_sum - budget_sum;
    let percentage = if budget_sum != 0.0 {
        (absolute / budget_sum) * 100.0
    } else {
        0.0
    };

    Ok(DeviationSummary {
        percentage,
        absolute,
    })
}

/// Deterministic status rules:
/// - deviation < 5% → Safe
/// - deviation 5-15% → Warning
/// - deviation > 15% → Critical
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
    use polars::prelude::*;

    /// LAYER 2 TEST 1: Status Threshold Boundaries (Exact)
    /// Verifies that status determination follows EXACT thresholds with no tolerance
    #[test]
    fn test_project_truth_status_thresholds() {
        // Test boundary at 5.0% (Safe/Warning threshold)
        let just_safe = DeviationSummary { percentage: 4.9, absolute: 100.0 };
        assert!(matches!(determine_status(&just_safe), ProjectStatus::Safe), 
            "4.9% must be Safe");

        let exactly_warning_start = DeviationSummary { percentage: 5.0, absolute: 100.0 };
        assert!(matches!(determine_status(&exactly_warning_start), ProjectStatus::Warning), 
            "5.0% must be Warning (inclusive lower bound)");

        // Test boundary at 15.0% (Warning/Critical threshold)
        let just_warning = DeviationSummary { percentage: 14.9, absolute: 500.0 };
        assert!(matches!(determine_status(&just_warning), ProjectStatus::Warning), 
            "14.9% must be Warning");

        let exactly_critical_start = DeviationSummary { percentage: 15.0, absolute: 500.0 };
        assert!(matches!(determine_status(&exactly_critical_start), ProjectStatus::Critical), 
            "15.0% must be Critical (inclusive lower bound)");

        // Test extremes
        let zero_deviation = DeviationSummary { percentage: 0.0, absolute: 0.0 };
        assert!(matches!(determine_status(&zero_deviation), ProjectStatus::Safe), 
            "0.0% must be Safe");

        let extreme_critical = DeviationSummary { percentage: 99.9, absolute: 10000.0 };
        assert!(matches!(determine_status(&extreme_critical), ProjectStatus::Critical), 
            "99.9% must be Critical");
    }

    /// LAYER 2 TEST 2: Financial Aggregation (Exact Arithmetic)
    /// Verifies that financial calculations produce exact results with no rounding errors
    #[test]
    fn test_financial_aggregation_exact() {
        // Create a known DataFrame with exact financial data
        let total_cost_col = Series::new("total_cost".into(), &[1000.50, 2500.75, 1500.25]);
        let total_paid_col = Series::new("total_paid".into(), &[500.00, 2500.75, 1000.00]);
        
        let df = DataFrame::new(vec![
            Column::from(total_cost_col),
            Column::from(total_paid_col),
        ]).expect("DataFrame creation should succeed");

        // Expected values (calculated manually):
        // total_cost: 1000.50 + 2500.75 + 1500.25 = 5001.50
        // total_paid: 500.00 + 2500.75 + 1000.00 = 4000.75
        // remaining: 5001.50 - 4000.75 = 1000.75

        let financials = calculate_financials(&df).expect("Financial calculation should succeed");

        // EXACT comparison - no tolerance
        assert_eq!(financials.total_cost, 5001.50, "Total cost must be exactly 5001.50");
        assert_eq!(financials.total_paid, 4000.75, "Total paid must be exactly 4000.75");
        assert_eq!(financials.remaining, 1000.75, "Remaining must be exactly 1000.75");
    }

    /// LAYER 2 TEST 3: Deviation Calculation (Exact Arithmetic)
    /// Verifies that deviation percentage is calculated with exact arithmetic
    #[test]
    fn test_deviation_calculation_exact() {
        // Create DataFrame with known budget vs actual values
        let budget_col = Series::new("budget".into(), &[1000.0, 2000.0, 3000.0]);
        let actual_col = Series::new("actual".into(), &[1100.0, 1900.0, 3300.0]);
        
        let df = DataFrame::new(vec![
            Column::from(budget_col),
            Column::from(actual_col),
        ]).expect("DataFrame creation should succeed");

        // Expected calculation:
        // Total budget: 1000 + 2000 + 3000 = 6000
        // Total actual: 1100 + 1900 + 3300 = 6300
        // Absolute deviation: 6300 - 6000 = 300
        // Percentage: (300 / 6000) * 100 = 5.0%

        let deviation = calculate_deviation(&df).expect("Deviation calculation should succeed");

        assert_eq!(deviation.absolute, 300.0, "Absolute deviation must be exactly 300.0");
        assert_eq!(deviation.percentage, 5.0, "Percentage deviation must be exactly 5.0%");
    }

    /// LAYER 2 TEST 4: Deterministic Calculation (Idempotence)
    /// Verifies that same DataFrame produces identical ProjectTruth every time
    #[test]
    fn test_derive_project_truth_deterministic() {
        let df = create_test_dataframe();
        let timestamp = "2026-01-31T13:00:00Z".to_string();

        let truth1 = derive_project_truth(&df, timestamp.clone()).expect("First derivation should succeed");
        let truth2 = derive_project_truth(&df, timestamp.clone()).expect("Second derivation should succeed");

        // Verify timestamp equality (deterministic)
        assert_eq!(truth1.last_updated, truth2.last_updated, "Timestamps must be identical");

        // Verify financial equality
        assert_eq!(truth1.financials.total_cost, truth2.financials.total_cost);
        assert_eq!(truth1.financials.total_paid, truth2.financials.total_paid);
        assert_eq!(truth1.financials.remaining, truth2.financials.remaining);

        // Verify deviation equality
        assert_eq!(truth1.deviation.percentage, truth2.deviation.percentage);
        assert_eq!(truth1.deviation.absolute, truth2.deviation.absolute);

        // Verify status equality
        assert_eq!(truth1.project_status, truth2.project_status);
        
        // Verify lineage equality
        assert_eq!(truth1.lineage.len(), truth2.lineage.len());
    }

    /// Helper: Create a test DataFrame with known values
    fn create_test_dataframe() -> DataFrame {
        let total_cost_col = Series::new("total_cost".into(), &[1000.0, 2000.0]);
        let total_paid_col = Series::new("total_paid".into(), &[900.0, 1800.0]);
        let budget_col = Series::new("budget".into(), &[1000.0, 2000.0]);
        let actual_col = Series::new("actual".into(), &[900.0, 1800.0]);
        
        let lin_cost = Series::new("_lineage_total_cost".into(), &["test_0_2", "test_1_2"]);
        let lin_paid = Series::new("_lineage_total_paid".into(), &["test_0_3", "test_1_3"]);
        
        DataFrame::new(vec![
            Column::from(total_cost_col),
            Column::from(total_paid_col),
            Column::from(budget_col),
            Column::from(actual_col),
            Column::from(lin_cost),
            Column::from(lin_paid),
        ]).expect("Test DataFrame creation should succeed")
    }
}
