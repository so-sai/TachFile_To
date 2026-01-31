use iron_table::contract::{TableTruth, TableRejection, CellValue};
use crate::diagnostics::CellRepair;
use crate::audit_log::AuditLogger;
use std::path::Path;

pub struct RepairEngine;

impl RepairEngine {
    /// Applies a batch of repairs to a TableTruth.
    /// Returns the repaired table or an error if the repairs themselves violate the foundational contract.
    pub fn apply_repairs(
        table: &TableTruth, 
        repairs: Vec<CellRepair>, 
        log_path: Option<&Path>
    ) -> Result<TableTruth, TableRejection> {
        let mut repaired_table = table.clone();
        
        for repair in &repairs {
            // Find the specific row and cell to repair
            let row = repaired_table.rows.iter_mut().find(|r| r.row_idx == repair.row_idx)
                .ok_or_else(|| TableRejection::ContractViolation(format!("Repair targets non-existent row {}", repair.row_idx)))?;
            
            let cell = row.cells.iter_mut().find(|c| c.col_idx == repair.col_idx)
                .ok_or_else(|| TableRejection::ContractViolation(format!("Repair targets non-existent col {} in row {}", repair.col_idx, repair.row_idx)))?;
            
            // Audit the repair before applying
            if let Some(path) = log_path {
                let details = serde_json::json!({
                    "row": repair.row_idx,
                    "col": repair.col_idx,
                    "old": repair.old_value,
                    "new": repair.new_value,
                    "reason": repair.reason
                });
                let _ = AuditLogger::log("REPAIR", &table.table_id, details, path);
            }

            // Apply the new value
            cell.value = repair.new_value.clone();
            
            // Mark as manually repaired
            cell.confidence = 1.0; 
            
            // SAFETY GATE: Verify that the human repair didn't inject Mojibake
            if let CellValue::Text(t) = &cell.value {
                if crate::gatekeeper::EncodingGatekeeper::is_mojibake(t) {
                    cell.encoding_status = iron_table::contract::EncodingStatus::Invalid;
                    cell.encoding_evidence = Some("Mojibake detected in human repair".into());
                } else {
                    cell.encoding_status = iron_table::contract::EncodingStatus::Clean;
                }
            } else {
                cell.encoding_status = iron_table::contract::EncodingStatus::Clean;
            }
        }
        
        // CRITICAL: Stateless Re-Validation
        // Repaired data must pass the exact same gate as extracted data.
        repaired_table.validate_contract()?;
        
        Ok(repaired_table)
    }

    /// Seals the current state into a TruthSnapshot.
    /// This is the "Niêm phong" operation.
    pub fn seal_truth(
        table_id: &str,
        raw_json_input: &str, 
        repairs: &Vec<CellRepair>, 
        virtual_truth: &TableTruth,
        actor: &str,
        verdict: &str
    ) -> crate::diagnostics::TruthSnapshot {
        use chrono::Local;
        use crate::diagnostics::{TruthSnapshot, HashSeal};

        let raw_hash = TruthSnapshot::calculate_hash(&raw_json_input);
        let repair_hash = TruthSnapshot::calculate_hash(repairs);
        let truth_hash = TruthSnapshot::calculate_hash(virtual_truth);

        TruthSnapshot {
            table_id: table_id.to_string(),
            hashes: HashSeal {
                raw_input: raw_hash,
                correction_batch: repair_hash,
                virtual_truth: truth_hash,
            },
            audited_by: actor.to_string(),
            verdict: verdict.to_string(),
            timestamp: Local::now().to_rfc3339(),
        }
    }
}
