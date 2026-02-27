//! Validation Engine: The Technical Verification Core.
//!
//! **Role:** Transforms technical signals into Data Verdicts (Rules R01-R05).
//! **Status:** SKELETON (Mission 034 - Refactored)
//!
//! This module DOES NOT judge business/legal compliance.
//! It verifies DATA CONSISTENCY only.

use iron_table::{DataVerdict, ProjectTruth, TableTruth, ViolationType};

pub struct ValidationContext<'a> {
    pub project_truth: &'a ProjectTruth,
    pub raw_tables: &'a [TableTruth],
    /// Pairs of tables to be cross-verified (e.g. PDF table vs Excel table)
    pub comparison_pairs: &'a [(&'a TableTruth, &'a TableTruth)],
}

// ============================================================================
// 2. THE VERIFIER (IMPL)
// ============================================================================

pub struct ValidationEngine;

impl ValidationEngine {
    /// The Main Verification Loop.
    /// Runs all technical checks on the data.
    pub fn verify(context: ValidationContext) -> Vec<DataVerdict> {
        let mut verdicts = Vec::new();

        // 1. Verify R01: Global Arithmetic Integrity
        verdicts.extend(Self::verify_r01_math(context.project_truth));

        // 2. Verify R05: Cross-Source Consistency (PDF vs Excel)
        for (primary, secondary) in context.comparison_pairs {
            verdicts.extend(Self::verify_r05_mismatch(primary, secondary));
        }

        verdicts
    }

    /// R01: Arithmetic Inconsistency
    fn verify_r01_math(truth: &ProjectTruth) -> Vec<DataVerdict> {
        let mut findings = Vec::new();

        // Example check: Remaining = Cost - Paid
        let calc_remaining = truth.financials.total_cost - truth.financials.total_paid;
        let diff = (truth.financials.remaining - calc_remaining).abs();

        if diff > 1.0 {
            // Epsilon 1.0 VND
            findings.push(DataVerdict {
                violation: ViolationType::MathError,
                severity: 3, // Critical
                message: format!(
                    "Inconsistent Financials: Calculated Remaining ({}) != Reported Remaining ({}) - Delta: {}", 
                    calc_remaining, truth.financials.remaining, diff
                ),
                technical_ref: "ProjectTruth::Financials".to_string(),
                evidence_id: None,
            });
        }

        findings
    }

    /// R05: Cross-Source Mismatch
    /// Compares two tables (e.g. PDF Scan vs Excel Import) cell-by-cell.
    /// Simplified logic: Assumes rows are aligned 1:1 by index for V1.
    fn verify_r05_mismatch(primary: &TableTruth, secondary: &TableTruth) -> Vec<DataVerdict> {
        let mut findings = Vec::new();

        let max_rows = std::cmp::min(primary.rows.len(), secondary.rows.len());

        for i in 0..max_rows {
            let row_p = &primary.rows[i];
            let row_s = &secondary.rows[i];

            // Compare cells if columns align (Naive V1 check)
            let max_cols = std::cmp::min(row_p.cells.len(), row_s.cells.len());

            for j in 0..max_cols {
                let cell_p = &row_p.cells[j];
                let cell_s = &row_s.cells[j];

                // Compare logic: Float vs Float
                if let (Some(val_p), Some(val_s)) =
                    (cell_p.value.as_float(), cell_s.value.as_float())
                {
                    let diff = (val_p - val_s).abs();
                    if diff > 1.0 {
                        // Epsilon 1.0 needed for cross-source too
                        findings.push(DataVerdict {
                            violation: ViolationType::Mismatch,
                            severity: 3,
                            message: format!(
                                "Cross-Source Mismatch: Source A ({}) vs Source B ({}) - Diff: {}",
                                val_p, val_s, diff
                            ),
                            technical_ref: format!("Row {}, Col {}", i, j),
                            evidence_id: Some(cell_p.global_id.clone()), // Point to Primary as truth reference
                        });
                    }
                }
            }
        }

        findings
    }
}
