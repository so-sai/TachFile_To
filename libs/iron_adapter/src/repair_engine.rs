use serde::{Deserialize, Serialize};
use iron_table::contract::{TableTruth, TableRejection, CellValue};
use crate::diagnostics::CellRepair;
use std::path::Path;

/// Encoding repair mode for legacy Vietnamese fonts
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LegacyEncodingMode {
    /// PRISTINE: Original UTF-8 data (no conversion)
    Unicode,
    /// VNI-Times, VNI-Helve font family (digit suffixes)
    Vni,
    /// TCVN3 / .VnTime font family (extended ASCII)
    Tcvn3,
    /// Auto-detect (tries VNI first, then TCVN3)
    Auto,
}

/// A candidate for encoding repair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodingCandidate {
    pub mode: LegacyEncodingMode,
    pub text: String,
}

pub struct RepairEngine;

impl RepairEngine {
    /// Applies a batch of repairs to a TableTruth.
    /// Returns the repaired table or an error if the repairs themselves violate the foundational contract.
    pub fn apply_repairs(
        table: &TableTruth, 
        repairs: Vec<CellRepair>, 
        _log_path: Option<&Path>
    ) -> Result<TableTruth, TableRejection> {
        let mut repaired_table = table.clone();
        
        for repair in &repairs {
            // Find the specific row and cell to repair
            let row = repaired_table.rows.iter_mut().find(|r| r.row_idx == repair.row_idx)
                .ok_or_else(|| TableRejection::ContractViolation(format!("Repair targets non-existent row {}", repair.row_idx)))?;
            
            let cell = row.cells.iter_mut().find(|c| c.col_idx == repair.col_idx)
                .ok_or_else(|| TableRejection::ContractViolation(format!("Repair targets non-existent col {} in row {}", repair.col_idx, repair.row_idx)))?;
            
            // 🛡️ POPULATE AUDIT FIELDS
            let mut repair_clone = repair.clone();
            repair_clone.timestamp = chrono::Local::now().to_rfc3339();
            repair_clone.actor = std::env::var("USERNAME").or_else(|_| std::env::var("USER")).unwrap_or_else(|_| "UNKNOWN_ACTOR".into());
            if repair_clone.reason_code.is_empty() {
                repair_clone.reason_code = "MANUAL_OVERRIDE".into();
            }
            
            // Generate individual repair hash
            repair_clone.signature_hash = crate::diagnostics::TruthSnapshot::calculate_hash(&repair_clone);

            // Apply the new value
            cell.value = repair_clone.new_value.clone();
            
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

    /// Convert legacy Vietnamese encoding to Unicode.
    /// 
    /// # Human-Gated Operation (LAW-07 Compliant)
    /// This function is intended to be called ONLY from the Human Repair Loop.
    /// The result must be reviewed by a human and applied via `apply_repairs()`.
    /// 
    /// # Returns
    /// A `CellRepair` struct ready to be applied, preserving full audit trail.
    pub fn normalize_legacy_encoding(
        row_idx: usize,
        col_idx: usize,
        original_text: &str,
        mode: LegacyEncodingMode,
    ) -> CellRepair {
        let normalizer = crate::encoding_normalizer::EncodingNormalizer::global();
        
        let normalized = match mode {
            LegacyEncodingMode::Unicode => original_text.to_string(),
            LegacyEncodingMode::Vni => normalizer.vni_to_unicode(original_text),
            LegacyEncodingMode::Tcvn3 => normalizer.tcvn3_to_unicode(original_text),
            LegacyEncodingMode::Auto => normalizer.auto_normalize(original_text),
        };

        let mode_name = match mode {
            LegacyEncodingMode::Unicode => "Unicode",
            LegacyEncodingMode::Vni => "VNI",
            LegacyEncodingMode::Tcvn3 => "TCVN3",
            LegacyEncodingMode::Auto => "Auto",
        };

        CellRepair {
            row_idx,
            col_idx,
            old_value: CellValue::Text(original_text.to_string()),
            new_value: CellValue::Text(normalized),
            reason: format!("Legacy encoding normalized ({} → Unicode)", mode_name),
            reason_code: "ENCODING_REPAIR".into(),
            timestamp: chrono::Local::now().to_rfc3339(),
            actor: std::env::var("USERNAME").or_else(|_| std::env::var("USER")).unwrap_or_else(|_| "SYSTEM".into()),
            signature_hash: String::new(), // To be generated on application
        }
    }

    /// Seals the current state into a TruthSnapshot.
    /// This is the "Niêm phong" operation.
    pub fn seal_truth(
        table_id: &str,
        raw_json_input: &str, 
        repairs: &Vec<CellRepair>, 
        virtual_truth: &TableTruth,
        actor: &str,
        verdict: &str,
        parent_hash: Option<String>
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
            repairs: repairs.clone(),
            audited_by: actor.to_string(),
            verdict: verdict.to_string(),
            timestamp: Local::now().to_rfc3339(),
            parent_hash,
        }
    }

    /// Returns a list of potential encoding interpretations for a given text.
    /// This is used for the "Auto-detect Preview" feature.
    pub fn get_encoding_candidates(input: &str) -> Vec<EncodingCandidate> {
        let normalizer = crate::encoding_normalizer::EncodingNormalizer::global();
        
        vec![
            EncodingCandidate {
                mode: LegacyEncodingMode::Unicode,
                text: input.to_string(),
            },
            EncodingCandidate {
                mode: LegacyEncodingMode::Vni,
                text: normalizer.vni_to_unicode(input),
            },
            EncodingCandidate {
                mode: LegacyEncodingMode::Tcvn3,
                text: normalizer.tcvn3_to_unicode(input),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_legacy_encoding_vni() {
        let repair = RepairEngine::normalize_legacy_encoding(
            0, 0,
            "ha2nh chi1nh",
            LegacyEncodingMode::Vni
        );
        
        assert_eq!(repair.row_idx, 0);
        assert_eq!(repair.col_idx, 0);
        assert!(repair.reason.contains("VNI"));
        
        if let CellValue::Text(new_val) = &repair.new_value {
            assert_eq!(new_val, "hành chính");
        } else {
            panic!("Expected Text value");
        }
    }

    #[test]
    fn test_normalize_legacy_encoding_preserves_audit_trail() {
        let repair = RepairEngine::normalize_legacy_encoding(
            5, 2,
            "test",
            LegacyEncodingMode::Auto
        );
        
        // Audit trail fields are populated
        assert_eq!(repair.row_idx, 5);
        assert_eq!(repair.col_idx, 2);
        assert!(matches!(repair.old_value, CellValue::Text(_)));
        assert!(matches!(repair.new_value, CellValue::Text(_)));
        assert!(!repair.reason.is_empty());
    }
}

