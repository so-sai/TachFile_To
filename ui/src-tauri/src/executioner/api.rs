/*
 * MISSION 012B - EXECUTIONER & QUIESCE PROTOCOL (API TYPES)
 * ==========================================================
 * Phase 1: API Definition (Frozen, Immutable)
 * 
 * This file contains all type definitions for the enforcement system.
 * These types are LOCKED and cannot be modified.
 */

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::resource_court::{EvictionVerdict, EvictionAction, CacheRegistry};

// ============================================================
// 1. EXECUTION WARRANT
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionWarrant {
    pub verdict: EvictionVerdict,
    pub nonce: u64,
    pub issued_at: u64,
    pub signature: String,
    pub ledger_ref: Option<String>,
}

impl ExecutionWarrant {
    pub fn new(verdict: EvictionVerdict, nonce: u64) -> Self {
        Self {
            verdict,
            nonce,
            issued_at: current_timestamp(),
            signature: String::new(),
            ledger_ref: None,
        }
    }

    pub fn is_valid(&self) -> bool {
        if self.nonce == 0 {
            return false;
        }
        if self.verdict.file_id.is_empty() {
            return false;
        }
        if self.issued_at == 0 {
            return false;
        }
        true
    }

    pub fn warrant_id(&self) -> String {
        format!("WARRANT_{:016x}", self.nonce)
    }
}

// ============================================================
// 2. EXECUTION REPORT
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub warrant_nonce: u64,
    pub file_id: String,
    pub action: EvictionAction,
    pub success: bool,
    pub error: Option<ExecutionError>,
    pub completed_at: u64,
    pub audit_detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionError {
    FileNotFound,
    PermissionDenied,
    IoError(String),
    FileLocked,
    WarrantAlreadyExecuted,
    WarrantNotInLedger,
    SystemQuiesced,
}

// ============================================================
// 3. QUIESCE SIGNAL
// ============================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum QuiesceSignal {
    None,
    Pending {
        file_id_hash: u64,
        deadline_unix_sec: u64,
    },
    Global {
        deadline_unix_sec: u64,
    },
}

impl QuiesceSignal {
    pub fn applies_to(&self, file_id_hash: u64) -> bool {
        match self {
            QuiesceSignal::None => false,
            QuiesceSignal::Pending {
                file_id_hash: target_hash,
                ..
            } => *target_hash == file_id_hash,
            QuiesceSignal::Global { .. } => true,
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = current_timestamp();
        match self {
            QuiesceSignal::None => false,
            QuiesceSignal::Pending {
                deadline_unix_sec, ..
            } => now > *deadline_unix_sec,
            QuiesceSignal::Global {
                deadline_unix_sec,
            } => now > *deadline_unix_sec,
        }
    }

    pub fn time_to_deadline(&self) -> Option<u64> {
        let now = current_timestamp();
        match self {
            QuiesceSignal::None => None,
            QuiesceSignal::Pending {
                deadline_unix_sec, ..
            } => {
                if now < *deadline_unix_sec {
                    Some(deadline_unix_sec - now)
                } else {
                    Some(0)
                }
            }
            QuiesceSignal::Global {
                deadline_unix_sec,
            } => {
                if now < *deadline_unix_sec {
                    Some(deadline_unix_sec - now)
                } else {
                    Some(0)
                }
            }
        }
    }
}

// ============================================================
// 4. NAMING CONTRACT
// ============================================================

#[derive(Debug)]
pub struct NamingContract;

impl NamingContract {
    pub fn validate(file_name: &str) -> (bool, Vec<String>) {
        let mut reasons = Vec::new();

        if !file_name.starts_with("TFT_") {
            reasons.push("Missing 'TFT_' prefix".to_string());
            return (false, reasons);
        }

        if !file_name.ends_with(".tft_cache") {
            reasons.push("Missing '.tft_cache' suffix".to_string());
            return (false, reasons);
        }

        let middle = file_name
            .strip_prefix("TFT_")
            .and_then(|s| s.strip_suffix(".tft_cache"));

        if let Some(parts_str) = middle {
            let parts: Vec<&str> = parts_str.split('_').collect();
            if parts.len() < 3 {
                reasons.push(
                    "Invalid format: need at least 3 underscore-separated parts".to_string(),
                );
                return (false, reasons);
            }

            if let Some(ts_part) = parts.get(2) {
                if !ts_part.chars().all(|c| c.is_numeric()) {
                    reasons.push("Timestamp part is not numeric".to_string());
                    return (false, reasons);
                }
            }

            (true, reasons)
        } else {
            reasons.push("Invalid structure".to_string());
            (false, reasons)
        }
    }

    pub fn classify(file_name: &str) -> FileOrigin {
        let (valid, _) = Self::validate(file_name);
        if valid {
            FileOrigin::Ghost
        } else {
            FileOrigin::Alien
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileOrigin {
    Ghost,
    Alien,
}

// ============================================================
// 5. SOFT DELETE SPEC
// ============================================================

pub struct SoftDeleteSpec {
    pub file_id: String,
    pub reason: String,
    pub is_reversible: bool,
}

// ============================================================
// 6. PROTOCOL 000
// ============================================================

pub struct PurgeAllProtocol {
    pub enabled: bool,
    pub phase: u8,
    pub targets: Vec<String>,
}

impl PurgeAllProtocol {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            phase: 0,
            targets: Vec::new(),
        }
    }

    pub fn phase_1_quiesce(&mut self, deadline_sec: u64) -> QuiesceSignal {
        self.phase = 1;
        QuiesceSignal::Global {
            deadline_unix_sec: current_timestamp() + deadline_sec,
        }
    }

    pub fn phase_2_collect_targets(&mut self, registry: &CacheRegistry) {
        self.phase = 2;
        self.targets = registry.entries().keys().cloned().collect();
    }

    pub fn phase_3_clear_registry(&mut self) -> usize {
        self.phase = 3;
        self.targets.len()
    }

    pub fn phase_4_execute(&mut self) -> Vec<ExecutionReport> {
        self.phase = 4;
        Vec::new()
    }
}

// ============================================================
// 7. LEDGER TYPES
// ============================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WarrantState {
    Pending,
    Executing,
    Committed,
    Failed,
    Revoked,
}

pub struct LedgerEntry {
    pub warrant_nonce: u64,
    pub state: WarrantState,
    pub created_at: u64,
    pub completed_at: Option<u64>,
    pub verdict: String,
    pub result: Option<ExecutionReport>,
}

// ============================================================
// 8. EXECUTIONER TRAIT
// ============================================================

pub trait Executioner {
    fn execute(
        &mut self,
        warrant: ExecutionWarrant,
    ) -> Result<ExecutionReport, ExecutionError>;
}

// ============================================================
// HELPER FUNCTIONS
// ============================================================

pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn hash_file_id(file_id: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in file_id.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_warrant_creation() {
        let verdict = EvictionVerdict {
            file_id: "test_file".to_string(),
            action: EvictionAction::SoftDelete,
            reason: "Test".to_string(),
            score: 0.7,
            timestamp: current_timestamp(),
            is_reversible: true,
        };

        let warrant = ExecutionWarrant::new(verdict, 12345);

        assert_eq!(warrant.nonce, 12345);
        assert!(warrant.is_valid());
        assert_eq!(warrant.warrant_id(), "WARRANT_0000000000003039");
    }

    #[test]
    fn test_quiesce_signal_expiration() {
        let now = current_timestamp();

        let signal_future = QuiesceSignal::Pending {
            file_id_hash: 42,
            deadline_unix_sec: now + 60,
        };

        assert!(!signal_future.is_expired());
        assert_eq!(signal_future.time_to_deadline(), Some(60));

        let signal_past = QuiesceSignal::Pending {
            file_id_hash: 42,
            deadline_unix_sec: now - 10,
        };

        assert!(signal_past.is_expired());
    }

    #[test]
    fn test_naming_contract_validation() {
        let valid_name = "TFT_a1b2c3d4_page_001_1234567890.tft_cache";
        let (valid, _) = NamingContract::validate(valid_name);
        assert!(valid);

        let invalid_prefix = "cache_a1b2c3d4_page_001_1234567890.tft_cache";
        let (valid, reasons) = NamingContract::validate(invalid_prefix);
        assert!(!valid);
        assert!(reasons.iter().any(|r| r.contains("TFT_")));

        let invalid_suffix = "TFT_a1b2c3d4_page_001_1234567890.tmp";
        let (valid, reasons) = NamingContract::validate(invalid_suffix);
        assert!(!valid);
        assert!(reasons.iter().any(|r| r.contains("tft_cache")));
    }

    #[test]
    fn test_file_origin_classification() {
        let ghost_file = "TFT_abc123_page_001_1609459200.tft_cache";
        assert_eq!(NamingContract::classify(ghost_file), FileOrigin::Ghost);

        let user_file = "my_important_document.pdf";
        assert_eq!(NamingContract::classify(user_file), FileOrigin::Alien);
    }

    #[test]
    fn test_quiesce_file_specific() {
        let file_id = "test_file_123";
        let file_hash = hash_file_id(file_id);

        let signal = QuiesceSignal::Pending {
            file_id_hash: file_hash,
            deadline_unix_sec: current_timestamp() + 60,
        };

        assert!(signal.applies_to(file_hash));
        assert!(!signal.applies_to(file_hash + 1));
    }

    #[test]
    fn test_quiesce_global_applies_to_all() {
        let signal = QuiesceSignal::Global {
            deadline_unix_sec: current_timestamp() + 60,
        };

        assert!(signal.applies_to(100));
        assert!(signal.applies_to(999));
        assert!(signal.applies_to(0));
    }
}
