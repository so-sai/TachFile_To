/*
 * LEDGER MODULE - SQLite Audit Trail
 * ==================================
 * 
 * Purpose: Single Source of Truth for all execution decisions
 * Mode: Append-only, STRICT, Sync-only (no async)
 * 
 * "Cuốn sổ cái công lý" - không ai có thể xóa, chỉ thêm
 */

use rusqlite::{Connection, params, OptionalExtension, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::resource_court::EvictionVerdict;

// ============================================================
// TYPES
// ============================================================

/// Entry in execution_warrants table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarrantEntry {
    pub nonce: String,           // TEXT PRIMARY KEY
    pub issued_at_unix: u64,     // INTEGER NOT NULL
    pub target_path: String,     // TEXT NOT NULL
    pub action: String,          // TEXT NOT NULL (SOFT_DELETE, HARD_DELETE)
    pub signature: Vec<u8>,      // BLOB NOT NULL
    pub court_version: String,   // TEXT NOT NULL (for future audits)
}

/// Entry in execution_events table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEventEntry {
    pub id: u32,                 // INTEGER PRIMARY KEY AUTOINCREMENT
    pub warrant_nonce: String,   // TEXT FK
    pub executed_at_unix: u64,   // INTEGER NOT NULL
    pub executor_id: String,     // TEXT NOT NULL
    pub result: String,          // TEXT (SUCCESS, FAIL_PERMISSION, FAIL_IO, FAIL_LOCKED)
    pub errno: Option<i32>,      // INTEGER (NULL if success)
}

/// Entry in system_events table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEventEntry {
    pub id: u32,                 // INTEGER PRIMARY KEY AUTOINCREMENT
    pub event_type: String,      // TEXT (QUIESCE_ENTER, QUIESCE_EXIT, PURGE_BEGIN, PURGE_END)
    pub issued_at_unix: u64,     // INTEGER NOT NULL
    pub deadline_unix: Option<u64>, // INTEGER (NULL if no deadline)
    pub actor: String,           // TEXT NOT NULL
}

/// Enum for result status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionResult {
    Success,
    FailPermission,
    FailIo,
    FailLocked,
}

impl ExecutionResult {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExecutionResult::Success => "SUCCESS",
            ExecutionResult::FailPermission => "FAIL_PERMISSION",
            ExecutionResult::FailIo => "FAIL_IO",
            ExecutionResult::FailLocked => "FAIL_LOCKED",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "SUCCESS" => Some(ExecutionResult::Success),
            "FAIL_PERMISSION" => Some(ExecutionResult::FailPermission),
            "FAIL_IO" => Some(ExecutionResult::FailIo),
            "FAIL_LOCKED" => Some(ExecutionResult::FailLocked),
            _ => None,
        }
    }
}

// ============================================================
// LEDGER TRAIT
// ============================================================

pub trait LedgerBackend {
    /// Append a new warrant to ledger (atomic)
    /// Returns the ledger_ref that can be stored in ExecutionWarrant
    fn append_warrant(&mut self, warrant: &WarrantEntry) -> SqliteResult<String>;

    /// Record an execution event
    fn record_execution(&mut self, event: &ExecutionEventEntry) -> SqliteResult<()>;

    /// Get all pending warrants (those with no SUCCESS event)
    fn get_pending_warrants(&self) -> SqliteResult<Vec<WarrantEntry>>;

    /// Check if warrant has been executed
    fn is_warrant_executed(&self, nonce: &str) -> SqliteResult<bool>;

    /// Record a system event (quiesce, purge, etc.)
    fn record_system_event(
        &mut self,
        event_type: &str,
        deadline_unix: Option<u64>,
        actor: &str,
    ) -> SqliteResult<()>;

    /// Verify ledger integrity (for startup checks)
    fn verify_integrity(&self) -> SqliteResult<()>;
}

// ============================================================
// SQLITE LEDGER IMPLEMENTATION
// ============================================================

pub struct SqliteLedger {
    conn: Connection,
}

impl SqliteLedger {
    /// Open or create ledger at given path
    pub fn open<P: AsRef<Path>>(path: P) -> SqliteResult<Self> {
        let conn = Connection::open(path)?;

        // Enable strict mode and WAL
        conn.execute_batch("PRAGMA journal_mode = WAL;")?;
        conn.execute_batch("PRAGMA synchronous = FULL;")?;

        // Create tables if not exist
        Self::init_schema(&conn)?;

        Ok(SqliteLedger { conn })
    }

    /// Create in-memory ledger (for testing)
    pub fn open_memory() -> SqliteResult<Self> {
        let conn = Connection::open_in_memory()?;

        conn.execute_batch("PRAGMA journal_mode = WAL;")?;
        conn.execute_batch("PRAGMA synchronous = FULL;")?;

        Self::init_schema(&conn)?;

        Ok(SqliteLedger { conn })
    }

    /// Initialize schema (STRICT mode, append-only)
    fn init_schema(conn: &Connection) -> SqliteResult<()> {
        // execution_warrants: immutable verdicts
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS execution_warrants (
                nonce TEXT PRIMARY KEY,
                issued_at_unix INTEGER NOT NULL,
                target_path TEXT NOT NULL,
                action TEXT NOT NULL CHECK(action IN ('SOFT_DELETE', 'HARD_DELETE')),
                signature BLOB NOT NULL,
                court_version TEXT NOT NULL
            ) STRICT;
            "#,
        )?;

        // execution_events: append-only record of attempts
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS execution_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                warrant_nonce TEXT NOT NULL,
                executed_at_unix INTEGER NOT NULL,
                executor_id TEXT NOT NULL,
                result TEXT NOT NULL CHECK(result IN ('SUCCESS', 'FAIL_PERMISSION', 'FAIL_IO', 'FAIL_LOCKED')),
                errno INTEGER,
                FOREIGN KEY (warrant_nonce) REFERENCES execution_warrants(nonce)
            ) STRICT;
            "#,
        )?;

        // system_events: quiesce, purge, etc.
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS system_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_type TEXT NOT NULL CHECK(event_type IN ('QUIESCE_ENTER', 'QUIESCE_EXIT', 'PURGE_BEGIN', 'PURGE_END')),
                issued_at_unix INTEGER NOT NULL,
                deadline_unix INTEGER,
                actor TEXT NOT NULL
            ) STRICT;
            "#,
        )?;

        // Create indexes (minimal, for audit only)
        conn.execute_batch(
            r#"
            CREATE INDEX IF NOT EXISTS idx_execution_events_nonce ON execution_events(warrant_nonce);
            CREATE INDEX IF NOT EXISTS idx_system_events_time ON system_events(issued_at_unix);
            "#,
        )?;

        Ok(())
    }

    /// Verify that all foreign keys exist and data is valid
    fn verify_integrity_impl(&self) -> SqliteResult<()> {
        // Check: all execution_events refer to valid warrants
        let mut stmt = self
            .conn
            .prepare("SELECT COUNT(*) FROM execution_events WHERE warrant_nonce NOT IN (SELECT nonce FROM execution_warrants)")?;

        let invalid_count: i32 = stmt.query_row([], |row| row.get(0))?;

        if invalid_count > 0 {
            return Err(rusqlite::Error::ExecuteReturnedResults);
        }

        Ok(())
    }
}

impl LedgerBackend for SqliteLedger {
    fn append_warrant(&mut self, warrant: &WarrantEntry) -> SqliteResult<String> {
        // Validate target_path matches Naming Contract (TFT_ prefix)
        // Accept either a logical id that starts with `TFT_` or a filesystem
        // path whose basename starts with `TFT_` (helps tests and real paths).
        let path = Path::new(&warrant.target_path);
        let name_ok = warrant.target_path.starts_with("TFT_")
            || path
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.starts_with("TFT_"))
                .unwrap_or(false);
        if !name_ok {
            return Err(rusqlite::Error::InvalidQuery);
        }

        // BEGIN IMMEDIATE: Fail-fast if another transaction is active
        self.conn.execute("BEGIN IMMEDIATE", [])?;

        let result = self.conn.execute(
            "INSERT INTO execution_warrants (nonce, issued_at_unix, target_path, action, signature, court_version)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &warrant.nonce,
                warrant.issued_at_unix,
                &warrant.target_path,
                &warrant.action,
                &warrant.signature,
                &warrant.court_version,
            ],
        );

        match result {
            Ok(_) => {
                self.conn.execute("COMMIT", [])?;
                // Return ledger reference (format: LE_<nonce>_<timestamp>)
                let now = current_timestamp();
                Ok(format!("LE_{}_{}", warrant.nonce, now))
            }
            Err(e) => {
                let _ = self.conn.execute("ROLLBACK", []);
                Err(e)
            }
        }
    }

    fn record_execution(&mut self, event: &ExecutionEventEntry) -> SqliteResult<()> {
        // Verify warrant exists
        let warrant_exists: bool = self.conn.query_row(
            "SELECT COUNT(*) FROM execution_warrants WHERE nonce = ?1",
            params![&event.warrant_nonce],
            |row| {
                let count: i32 = row.get(0)?;
                Ok(count > 0)
            },
        )?;

        if !warrant_exists {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        // BEGIN IMMEDIATE: Atomic write
        self.conn.execute("BEGIN IMMEDIATE", [])?;

        let result = self.conn.execute(
            "INSERT INTO execution_events (warrant_nonce, executed_at_unix, executor_id, result, errno)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                &event.warrant_nonce,
                event.executed_at_unix,
                &event.executor_id,
                &event.result,
                event.errno,
            ],
        );

        match result {
            Ok(_) => {
                self.conn.execute("COMMIT", [])?;
                Ok(())
            }
            Err(e) => {
                let _ = self.conn.execute("ROLLBACK", []);
                Err(e)
            }
        }
    }

    fn get_pending_warrants(&self) -> SqliteResult<Vec<WarrantEntry>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT w.nonce, w.issued_at_unix, w.target_path, w.action, w.signature, w.court_version
            FROM execution_warrants w
            WHERE w.nonce NOT IN (
                SELECT warrant_nonce FROM execution_events WHERE result = 'SUCCESS'
            )
            "#,
        )?;

        let warrants = stmt.query_map([], |row| {
            Ok(WarrantEntry {
                nonce: row.get(0)?,
                issued_at_unix: row.get(1)?,
                target_path: row.get(2)?,
                action: row.get(3)?,
                signature: row.get(4)?,
                court_version: row.get(5)?,
            })
        })?;

        let mut result = Vec::new();
        for warrant in warrants {
            result.push(warrant?);
        }

        Ok(result)
    }

    fn is_warrant_executed(&self, nonce: &str) -> SqliteResult<bool> {
        let executed = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM execution_events WHERE warrant_nonce = ?1 AND result = 'SUCCESS'",
                params![nonce],
                |row| {
                    let count: i32 = row.get(0)?;
                    Ok(count > 0)
                },
            )
            .optional()?
            .unwrap_or(false);

        Ok(executed)
    }

    fn record_system_event(
        &mut self,
        event_type: &str,
        deadline_unix: Option<u64>,
        actor: &str,
    ) -> SqliteResult<()> {
        // Validate event_type
        if !["QUIESCE_ENTER", "QUIESCE_EXIT", "PURGE_BEGIN", "PURGE_END"].contains(&event_type) {
            return Err(rusqlite::Error::InvalidQuery);
        }

        self.conn.execute("BEGIN IMMEDIATE", [])?;

        let result = self.conn.execute(
            "INSERT INTO system_events (event_type, issued_at_unix, deadline_unix, actor)
             VALUES (?1, ?2, ?3, ?4)",
            params![event_type, current_timestamp(), deadline_unix, actor],
        );

        match result {
            Ok(_) => {
                self.conn.execute("COMMIT", [])?;
                Ok(())
            }
            Err(e) => {
                let _ = self.conn.execute("ROLLBACK", []);
                Err(e)
            }
        }
    }

    fn verify_integrity(&self) -> SqliteResult<()> {
        self.verify_integrity_impl()
    }
}

// ============================================================
// HELPER FUNCTIONS
// ============================================================

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ledger_open_memory() {
        let ledger = SqliteLedger::open_memory().expect("Failed to open ledger");
        ledger.verify_integrity().expect("Ledger integrity check failed");
    }

    #[test]
    fn test_append_warrant() {
        let mut ledger = SqliteLedger::open_memory().expect("Failed to open ledger");

        let warrant = WarrantEntry {
            nonce: "test_nonce_001".to_string(),
            issued_at_unix: 1000,
            target_path: "TFT_abc123_page_001_1000.tft_cache".to_string(),
            action: "SOFT_DELETE".to_string(),
            signature: vec![0xAB, 0xCD],
            court_version: "1.0".to_string(),
        };

        let ledger_ref = ledger
            .append_warrant(&warrant)
            .expect("Failed to append warrant");

        assert!(ledger_ref.starts_with("LE_test_nonce_001"));

        // Verify it was written
        let pending = ledger
            .get_pending_warrants()
            .expect("Failed to get pending warrants");
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].nonce, "test_nonce_001");
    }

    #[test]
    fn test_reject_invalid_path() {
        let mut ledger = SqliteLedger::open_memory().expect("Failed to open ledger");

        let invalid_warrant = WarrantEntry {
            nonce: "test_nonce_002".to_string(),
            issued_at_unix: 1000,
            target_path: "invalid_path.tmp".to_string(), // ❌ No TFT_ prefix
            action: "HARD_DELETE".to_string(),
            signature: vec![0xAB, 0xCD],
            court_version: "1.0".to_string(),
        };

        let result = ledger.append_warrant(&invalid_warrant);
        assert!(result.is_err(), "Should reject warrant without TFT_ prefix");
    }

    #[test]
    fn test_execution_event_idempotence() {
        let mut ledger = SqliteLedger::open_memory().expect("Failed to open ledger");

        // Append warrant
        let warrant = WarrantEntry {
            nonce: "test_nonce_003".to_string(),
            issued_at_unix: 2000,
            target_path: "TFT_def456_page_002_2000.tft_cache".to_string(),
            action: "HARD_DELETE".to_string(),
            signature: vec![0xDE, 0xF0],
            court_version: "1.0".to_string(),
        };

        ledger
            .append_warrant(&warrant)
            .expect("Failed to append warrant");

        // Record execution
        let event = ExecutionEventEntry {
            id: 0,
            warrant_nonce: "test_nonce_003".to_string(),
            executed_at_unix: 2100,
            executor_id: "executor_1".to_string(),
            result: "SUCCESS".to_string(),
            errno: None,
        };

        ledger
            .record_execution(&event)
            .expect("Failed to record execution");

        // Verify warrant is no longer pending
        let pending = ledger
            .get_pending_warrants()
            .expect("Failed to get pending warrants");
        assert_eq!(pending.len(), 0, "Executed warrant should not be pending");

        // Verify can query execution status
        let is_executed = ledger
            .is_warrant_executed("test_nonce_003")
            .expect("Failed to check execution");
        assert!(is_executed);
    }

    #[test]
    fn test_system_events() {
        let mut ledger = SqliteLedger::open_memory().expect("Failed to open ledger");

        let deadline = current_timestamp() + 30;
        ledger
            .record_system_event("QUIESCE_ENTER", Some(deadline), "court_1")
            .expect("Failed to record quiesce enter");

        ledger
            .record_system_event("QUIESCE_EXIT", None, "court_1")
            .expect("Failed to record quiesce exit");

        // If we can record without panic, system events are working
        assert!(true);
    }

    #[test]
    fn test_verify_integrity() {
        let ledger = SqliteLedger::open_memory().expect("Failed to open ledger");
        ledger
            .verify_integrity()
            .expect("Integrity check should pass for empty ledger");
    }

    #[test]
    fn test_invalid_action_rejected() {
        let mut ledger = SqliteLedger::open_memory().expect("Failed to open ledger");

        let invalid_warrant = WarrantEntry {
            nonce: "test_nonce_004".to_string(),
            issued_at_unix: 3000,
            target_path: "TFT_xyz789_page_003_3000.tft_cache".to_string(),
            action: "INVALID_ACTION".to_string(), // ❌ Not in CHECK constraint
            signature: vec![0x12, 0x34],
            court_version: "1.0".to_string(),
        };

        let result = ledger.append_warrant(&invalid_warrant);
        assert!(
            result.is_err(),
            "Should reject warrant with invalid action"
        );
    }
}
