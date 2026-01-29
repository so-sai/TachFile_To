/*
 * FILESYSTEM EXECUTOR - MECHANICAL ARM OF ENFORCEMENT
 * =====================================================
 * 
 * Phase 3.1: FilesystemExecutioner
 * 
 * This is the ONLY place where fs::remove_file() is called.
 * It has exactly ONE job: execute what Ledger says.
 * It has NO authority to question Ledger decisions.
 * 
 * Constraint: "Đao phủ không được suy nghĩ, chỉ được thi hành"
 * (Executioner cannot think, only execute)
 * 
 * Architecture: Uses Exclusive Reference (&'a mut L) to borrow Ledger
 * This ensures Ledger is the single source of truth without duplication.
 */

use crate::executioner::api::{
    ExecutionWarrant, ExecutionReport, ExecutionError, Executioner, current_timestamp,
};
use crate::executioner::ledger::{
    LedgerBackend, ExecutionEventEntry, ExecutionResult, WarrantEntry,
};
use crate::resource_court::EvictionAction;
use std::fs;
use std::io;
use std::path::PathBuf;

// ============================================================
// FILESYSTEM EXECUTOR
// ============================================================

pub struct FilesystemExecutioner<'a, L: LedgerBackend> {
    /// Exclusive mutable reference to ledger (borrowed, not owned)
    /// Ensures Ledger is the single source of truth
    ledger: &'a mut L,
    /// Identifier for this executor instance (for logging)
    executor_id: String,
}

impl<'a, L: LedgerBackend> FilesystemExecutioner<'a, L> {
    /// Create new executor that borrows a ledger
    pub fn new(ledger: &'a mut L, executor_id: String) -> Self {
        Self { ledger, executor_id }
    }

    /// Internal: execute soft delete (Registry-only, no filesystem)
    fn execute_soft_delete(&self, warrant: &ExecutionWarrant) -> Result<(), ExecutionError> {
        // Soft delete is purely logical: remove from Registry
        // No filesystem I/O happens here
        // This would be called by the Registry layer
        // For now, we just log that we're executing soft delete
        
        log_action(
            &warrant.verdict.file_id,
            warrant.nonce,
            "SOFT_DELETE_REQUESTED",
        );
        Ok(())
    }

    /// Internal: execute hard delete (filesystem removal)
    fn execute_hard_delete(&self, warrant: &ExecutionWarrant) -> Result<(), io::Error> {
        let file_path = PathBuf::from(&warrant.verdict.file_id);
        
        // Attempt to remove the file
        fs::remove_file(&file_path)?;
        
        log_action(
            &warrant.verdict.file_id,
            warrant.nonce,
            "HARD_DELETE_SUCCESS",
        );
        Ok(())
    }
}

impl<'a, L: LedgerBackend> Executioner for FilesystemExecutioner<'a, L> {
    fn execute(
        &mut self,
        warrant: ExecutionWarrant,
    ) -> Result<ExecutionReport, ExecutionError> {
        let nonce = warrant.nonce;
        let file_id = warrant.verdict.file_id.clone();
        let action = &warrant.verdict.action;
        
        log_action(&file_id, nonce, "EXECUTION_START");

        // ============================================================
        // STEP 1: Idempotence Check
        // ============================================================
        // If already executed, return success (idempotent operation)
        match self.ledger.is_warrant_executed(&warrant.warrant_id()) {
            Ok(true) => {
                log_action(&file_id, nonce, "ALREADY_EXECUTED (IDEMPOTENT)");
                return Ok(ExecutionReport {
                    warrant_nonce: nonce,
                    file_id,
                    action: action.clone(),
                    success: true,
                    error: None,
                    completed_at: current_timestamp(),
                    audit_detail: Some("Idempotent re-execution".to_string()),
                });
            }
            Ok(false) => {
                // Warrant not executed yet — verify it exists in ledger (pending warrants)
                match self.ledger.get_pending_warrants() {
                    Ok(pending) => {
                        if !pending.iter().any(|w| w.nonce == warrant.warrant_id()) {
                            log_action_error(
                                &file_id,
                                nonce,
                                "IDEMPOTENCE_CHECK",
                                "WarrantMissing",
                                None,
                            );
                            return Err(ExecutionError::WarrantNotInLedger);
                        }
                    }
                    Err(_) => {
                        // Ledger failure is a STOP condition
                        log_action_error(&file_id, nonce, "IDEMPOTENCE_CHECK", "LedgerError", None);
                        return Err(ExecutionError::WarrantNotInLedger);
                    }
                }
            }
            Err(_) => {
                // Ledger failure is a STOP condition
                log_action_error(&file_id, nonce, "IDEMPOTENCE_CHECK", "LedgerError", None);
                return Err(ExecutionError::WarrantNotInLedger);
            }
        }

        // ============================================================
        // STEP 2: Execute Action
        // ============================================================
        let (execution_result, errno) = match action {
            EvictionAction::SoftDelete => {
                match self.execute_soft_delete(&warrant) {
                    Ok(_) => {
                        log_action(&file_id, nonce, "SOFT_DELETE_EXECUTED");
                        (ExecutionResult::Success, None)
                    }
                    Err(_) => {
                        log_action_error(&file_id, nonce, "SOFT_DELETE", "Failed", None);
                        (ExecutionResult::FailIo, None)
                    }
                }
            }
            EvictionAction::HardDelete => {
                match self.execute_hard_delete(&warrant) {
                    Ok(_) => {
                        log_action(&file_id, nonce, "HARD_DELETE_SUCCESS");
                        (ExecutionResult::Success, None)
                    }
                    Err(io_err) => {
                        let errno_val = io_err.raw_os_error();
                        // File not found is SUCCESS in hard delete context (idempotent)
                        if io_err.kind() == io::ErrorKind::NotFound {
                            log_action(&file_id, nonce, "HARD_DELETE_SUCCESS (NOTFOUND)");
                            (ExecutionResult::Success, errno_val)
                        } else if io_err.kind() == io::ErrorKind::PermissionDenied {
                            log_action_error(&file_id, nonce, "HARD_DELETE", "PermissionDenied", errno_val);
                            (ExecutionResult::FailPermission, errno_val)
                        } else {
                            log_action_error(&file_id, nonce, "HARD_DELETE", "IoError", errno_val);
                            (ExecutionResult::FailIo, errno_val)
                        }
                    }
                }
            }
            _ => {
                log_action_error(&file_id, nonce, "EXECUTE", "UnknownAction", None);
                return Err(ExecutionError::IoError(format!(
                    "Unknown action: {:?}",
                    action
                )));
            }
        };

        // ============================================================
        // STEP 3: Record to Ledger
        // ============================================================
        let event = ExecutionEventEntry {
            id: 0, // Auto-assigned by DB
            warrant_nonce: warrant.warrant_id(),
            executed_at_unix: current_timestamp(),
            executor_id: self.executor_id.clone(),
            result: execution_result.as_str().to_string(),
            errno,
        };

        if let Err(ledger_err) = self.ledger.record_execution(&event) {
            log_action_error(&file_id, nonce, "RECORD_EXECUTION", "LedgerError", None);
            return Err(ExecutionError::IoError(format!(
                "Failed to record execution: {}",
                ledger_err
            )));
        }

        log_action(&file_id, nonce, "LEDGER_RECORDED");

        // ============================================================
        // STEP 4: Return Report
        // ============================================================
        let success = execution_result == ExecutionResult::Success;
        let error = if success {
            None
        } else {
            match execution_result {
                ExecutionResult::Success => None,
                ExecutionResult::FailPermission => Some(ExecutionError::PermissionDenied),
                ExecutionResult::FailIo => {
                    Some(ExecutionError::IoError("I/O operation failed".to_string()))
                }
                ExecutionResult::FailLocked => Some(ExecutionError::FileLocked),
            }
        };

        log_action(&file_id, nonce, "EXECUTION_COMPLETE");

        Ok(ExecutionReport {
            warrant_nonce: nonce,
            file_id,
            action: action.clone(),
            success,
            error,
            completed_at: current_timestamp(),
            audit_detail: Some(format!(
                "Executor: {}, Result: {}",
                self.executor_id,
                execution_result.as_str()
            )),
        })
    }
}

// ============================================================
// LOGGING UTILITIES
// ============================================================

/// Log an action with warrant nonce (required for audit)
fn log_action(file_id: &str, nonce: u64, action: &str) {
    eprintln!(
        "[EXECUTOR] NONCE={:016x} FILE_ID={} ACTION={}",
        nonce, file_id, action
    );
}

/// Log an error with warrant nonce
fn log_action_error(
    file_id: &str,
    nonce: u64,
    action: &str,
    error_type: &str,
    errno: Option<i32>,
) {
    if let Some(err_code) = errno {
        eprintln!(
            "[EXECUTOR] NONCE={:016x} FILE_ID={} ACTION={} ERROR={} ERRNO={}",
            nonce, file_id, action, error_type, err_code
        );
    } else {
        eprintln!(
            "[EXECUTOR] NONCE={:016x} FILE_ID={} ACTION={} ERROR={}",
            nonce, file_id, action, error_type
        );
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executioner::ledger::SqliteLedger;
    use crate::resource_court::{EvictionVerdict, EvictionAction};
    use tempfile;
    use std::io::Write;

    /// Helper: create a test executor with in-memory ledger
    fn create_test_executor<'a>(ledger: &'a mut SqliteLedger) -> FilesystemExecutioner<'a, SqliteLedger> {
        FilesystemExecutioner::new(ledger, "test_executor".to_string())
    }

    #[test]
    fn test_execute_hard_delete_success() {
        let mut ledger = SqliteLedger::open_memory().expect("Failed to open in-memory ledger");

        // Create a temporary directory and a TFT_ named file inside it
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let file_path_buf = temp_dir.path().join("TFT_tempfile_exec.tft_cache");
        let mut temp_file = std::fs::File::create(&file_path_buf).expect("Failed to create temp file");
        temp_file
            .write_all(b"test content")
            .expect("Failed to write to temp file");
        let file_path = file_path_buf.to_string_lossy().to_string();
        
        // Verify file exists
        assert!(
            std::path::Path::new(&file_path).exists(),
            "Temp file should exist before deletion"
        );

        // Create warrant
        let verdict = EvictionVerdict {
            file_id: file_path.clone(),
            action: EvictionAction::HardDelete,
            reason: "test".to_string(),
            score: 100.0,
            timestamp: current_timestamp(),
            is_reversible: false,
        };
        let warrant = ExecutionWarrant::new(verdict, 12345);

        // Record warrant in ledger first
        let warrant_entry = WarrantEntry {
            nonce: warrant.warrant_id(),
            issued_at_unix: current_timestamp(),
            target_path: file_path.clone(),
            action: "HARD_DELETE".to_string(),
            signature: vec![],
            court_version: "1.0".to_string(),
        };
        ledger
            .append_warrant(&warrant_entry)
            .expect("Failed to append warrant");

        // Create executor after ledger has the warrant
        let mut executor = create_test_executor(&mut ledger);

        // Execute
        let result = executor.execute(warrant).expect("Execution should succeed");

        // Verify result
        assert!(result.success, "Hard delete should succeed");
        assert!(result.error.is_none(), "Should have no error");
        
        // Verify file is deleted
        assert!(
            !std::path::Path::new(&file_path).exists(),
            "File should be deleted after execution"
        );
    }

    #[test]
    fn test_execute_hard_delete_not_found_is_success() {
        let mut ledger = SqliteLedger::open_memory().expect("Failed to open in-memory ledger");

        // Try to delete a file that doesn't exist (but with TFT_ basename)
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let file_path_buf = temp_dir.path().join("TFT_nonexistent_file.tft_cache");
        let file_path = file_path_buf.to_string_lossy().to_string();

        // Create warrant
        let verdict = EvictionVerdict {
            file_id: file_path.clone(),
            action: EvictionAction::HardDelete,
            reason: "test".to_string(),
            score: 100.0,
            timestamp: current_timestamp(),
            is_reversible: false,
        };
        let warrant = ExecutionWarrant::new(verdict, 12346);

        // Record warrant in ledger
        let warrant_entry = WarrantEntry {
            nonce: warrant.warrant_id(),
            issued_at_unix: current_timestamp(),
            target_path: file_path.clone(),
            action: "HARD_DELETE".to_string(),
            signature: vec![],
            court_version: "1.0".to_string(),
        };
        ledger
            .append_warrant(&warrant_entry)
            .expect("Failed to append warrant");

        // Create executor after ledger has the warrant
        let mut executor = create_test_executor(&mut ledger);

        // Execute - should treat "not found" as success
        let result = executor.execute(warrant).expect("Execution should succeed");

        // Even if file doesn't exist, result should be success (idempotent)
        assert!(
            result.success,
            "Hard delete of non-existent file should be success (idempotent)"
        );
    }

    #[test]
    fn test_execute_soft_delete() {
        let mut ledger = SqliteLedger::open_memory().expect("Failed to open in-memory ledger");

        let file_path = "TFT_cached_file".to_string();

        let verdict = EvictionVerdict {
            file_id: file_path.clone(),
            action: EvictionAction::SoftDelete,
            reason: "test".to_string(),
            score: 50.0,
            timestamp: current_timestamp(),
            is_reversible: true,
        };
        let warrant = ExecutionWarrant::new(verdict, 12347);

        // Record warrant in ledger
        let warrant_entry = WarrantEntry {
            nonce: warrant.warrant_id(),
            issued_at_unix: current_timestamp(),
            target_path: file_path.clone(),
            action: "SOFT_DELETE".to_string(),
            signature: vec![],
            court_version: "1.0".to_string(),
        };
        ledger
            .append_warrant(&warrant_entry)
            .expect("Failed to append warrant");

        // Create executor after ledger has the warrant
        let mut executor = create_test_executor(&mut ledger);

        // Execute
        let result = executor.execute(warrant).expect("Execution should succeed");

        assert!(result.success, "Soft delete should succeed");
        assert!(result.error.is_none(), "Should have no error");
    }

    #[test]
    fn test_execute_idempotent() {
        let mut ledger = SqliteLedger::open_memory().expect("Failed to open in-memory ledger");

        let file_path = "TFT_path_to_file".to_string();

        let verdict = EvictionVerdict {
            file_id: file_path.clone(),
            action: EvictionAction::SoftDelete,
            reason: "test".to_string(),
            score: 50.0,
            timestamp: current_timestamp(),
            is_reversible: true,
        };
        let warrant = ExecutionWarrant::new(verdict, 12348);

        // Record warrant in ledger
        let warrant_entry = WarrantEntry {
            nonce: warrant.warrant_id(),
            issued_at_unix: current_timestamp(),
            target_path: file_path.clone(),
            action: "SOFT_DELETE".to_string(),
            signature: vec![],
            court_version: "1.0".to_string(),
        };
        ledger
            .append_warrant(&warrant_entry)
            .expect("Failed to append warrant");

        // Record an execution event (simulating first execution)
        let event = ExecutionEventEntry {
            id: 0,
            warrant_nonce: warrant.warrant_id(),
            executed_at_unix: current_timestamp(),
            executor_id: "test_executor".to_string(),
            result: "SUCCESS".to_string(),
            errno: None,
        };
        ledger
            .record_execution(&event)
            .expect("Failed to record execution");

        // Create executor after ledger has warrant and event
        let mut executor = create_test_executor(&mut ledger);

        // Execute again - should be idempotent
        let result = executor.execute(warrant).expect("Execution should succeed");

        assert!(result.success, "Second execution should succeed (idempotent)");
        assert_eq!(
            result.audit_detail.unwrap(),
            "Idempotent re-execution",
            "Should indicate idempotent execution"
        );
    }

    #[test]
    fn test_execute_warrant_not_in_ledger_fails() {
        let mut ledger = SqliteLedger::open_memory().expect("Failed to open in-memory ledger");
        let mut executor = create_test_executor(&mut ledger);

        let file_path = "TFT_path_to_file".to_string();

        let verdict = EvictionVerdict {
            file_id: file_path,
            action: EvictionAction::SoftDelete,
            reason: "test".to_string(),
            score: 50.0,
            timestamp: current_timestamp(),
            is_reversible: true,
    };
        let warrant = ExecutionWarrant::new(verdict, 12349);

        // DON'T record warrant in ledger

        // Execute - should fail because warrant not in ledger
        let result = executor.execute(warrant);

        assert!(result.is_err(), "Execution should fail if warrant not in ledger");
        match result {
            Err(ExecutionError::WarrantNotInLedger) => {} // Expected
            _ => panic!("Expected WarrantNotInLedger error"),
        }
    }
}
