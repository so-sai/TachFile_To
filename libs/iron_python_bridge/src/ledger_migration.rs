//! # Ledger Migration Module
//!
//! SQLite-based persistence for extracted documents.
//! Implements idempotent INSERT OR REPLACE pattern.

use anyhow::{Context, Result as AnyResult};
use chrono::{DateTime, FixedOffset, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{error, info, warn};

/// Ledger entry for extracted document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id: String,                                // Unique document ID (SHA-256)
    pub source_path: String,                       // Original file path
    pub checksum: String,                          // File checksum
    pub pages_processed: i32,                      // Number of pages processed
    pub extraction_engine: String,                 // "docling" or "mupdf"
    pub extraction_version: String,                // Engine version
    pub processing_time_ms: i64,                   // Processing duration
    pub status: String,                            // "success", "failed", "partial"
    pub error_message: Option<String>,             // Error if any
    pub metadata_json: String,                     // Serialized IngestionResult metadata
    pub created_at: DateTime<chrono::FixedOffset>, // Timestamp
}

/// SQLite ledger manager
pub struct LedgerManager {
    conn: Connection,
}

impl LedgerManager {
    /// Initialize ledger with proper schema
    pub fn new(db_path: &Path) -> AnyResult<Self> {
        let conn = Connection::open(db_path).context("Failed to open SQLite ledger database")?;

        // Create table with proper schema
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS ingestion_ledger (
                id TEXT PRIMARY KEY,
                source_path TEXT NOT NULL,
                checksum TEXT NOT NULL,
                pages_processed INTEGER NOT NULL,
                extraction_engine TEXT NOT NULL,
                extraction_version TEXT NOT NULL,
                processing_time_ms INTEGER NOT NULL,
                status TEXT NOT NULL,
                error_message TEXT,
                metadata_json TEXT NOT NULL,
                created_at TEXT NOT NULL
            )
            "#,
            [],
        )
        .context("Failed to create ingestion_ledger table")?;

        // Create indexes for performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_checksum ON ingestion_ledger(checksum)",
            [],
        )
        .context("Failed to create checksum index")?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_status ON ingestion_ledger(status)",
            [],
        )
        .context("Failed to create status index")?;

        info!("Ledger initialized at: {}", db_path.display());

        Ok(Self { conn })
    }

    /// Migrate extraction result to ledger (idempotent)
    pub fn migrate_to_ledger(&self, entry: LedgerEntry) -> AnyResult<()> {
        info!("Migrating document to ledger: {}", entry.id);

        let result = self.conn.execute(
            r#"
            INSERT OR REPLACE INTO ingestion_ledger 
            (id, source_path, checksum, pages_processed, extraction_engine, 
             extraction_version, processing_time_ms, status, error_message, 
             metadata_json, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            params![
                entry.id,
                entry.source_path,
                entry.checksum,
                entry.pages_processed,
                entry.extraction_engine,
                entry.extraction_version,
                entry.processing_time_ms,
                entry.status,
                entry.error_message,
                entry.metadata_json,
                entry.created_at.to_rfc3339()
            ],
        );

        match result {
            Ok(rows_affected) => {
                if rows_affected == 1 {
                    info!("Ledger entry created/updated: {}", entry.id);
                } else {
                    warn!(
                        "Unexpected row count during ledger migration: {}",
                        rows_affected
                    );
                }
            }
            Err(e) => {
                error!("Failed to migrate to ledger: {}", e);
                return Err(anyhow::anyhow!("Ledger migration failed: {}", e));
            }
        }

        Ok(())
    }

    /// Get entry by checksum
    pub fn get_by_checksum(&self, checksum: &str) -> AnyResult<Option<LedgerEntry>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, source_path, checksum, pages_processed, extraction_engine, 
                    extraction_version, processing_time_ms, status, error_message, 
                    metadata_json, created_at 
             FROM ingestion_ledger WHERE checksum = ?",
            )
            .context("Failed to prepare get_by_checksum query")?;

        let entry = stmt.query_row([checksum], |row| {
            Ok(LedgerEntry {
                id: row.get(0)?,
                source_path: row.get(1)?,
                checksum: row.get(2)?,
                pages_processed: row.get(3)?,
                extraction_engine: row.get(4)?,
                extraction_version: row.get(5)?,
                processing_time_ms: row.get(6)?,
                status: row.get(7)?,
                error_message: row.get(8)?,
                metadata_json: row.get(9)?,
                created_at: DateTime::parse_from_rfc3339(row.get::<_, String>(10)?.as_str())
                    .unwrap_or_else(|_| {
                        let utc_dt = DateTime::from_timestamp(0, 0).unwrap();
                        DateTime::from_naive_utc_and_offset(
                            utc_dt.naive_utc(),
                            FixedOffset::east_opt(0).unwrap(),
                        )
                    })
                    .into(),
            })
        });

        match entry {
            Ok(entry) => Ok(Some(entry)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("Failed to query ledger: {}", e)),
        }
    }

    /// Get statistics
    pub fn get_stats(&self) -> AnyResult<LedgerStats> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT 
                COUNT(*) as total_entries,
                COUNT(CASE WHEN status = 'success' THEN 1 END) as successful,
                COUNT(CASE WHEN status = 'failed' THEN 1 END) as failed,
                AVG(processing_time_ms) as avg_processing_time,
                MAX(created_at) as last_processed
             FROM ingestion_ledger",
            )
            .context("Failed to prepare stats query")?;

        let stats = stmt.query_row([], |row| {
            Ok(LedgerStats {
                total_entries: row.get(0)?,
                successful: row.get(1)?,
                failed: row.get(2)?,
                avg_processing_time_ms: row.get::<_, Option<f64>>(3)?,
                last_processed: row.get::<_, Option<String>>(4)?,
            })
        })?;

        Ok(stats)
    }

    /// Close ledger connection
    pub fn close(&self) -> AnyResult<()> {
        info!("Closing ledger connection");
        Ok(())
    }
}

/// Ledger statistics
#[derive(Debug, Clone)]
pub struct LedgerStats {
    pub total_entries: i64,
    pub successful: i64,
    pub failed: i64,
    pub avg_processing_time_ms: Option<f64>,
    pub last_processed: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_ledger_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_ledger.db");

        let ledger = LedgerManager::new(&db_path);
        assert!(ledger.is_ok());

        let ledger = ledger.unwrap();
        let stats = ledger.get_stats();
        assert!(stats.is_ok());

        let stats = stats.unwrap();
        assert_eq!(stats.total_entries, 0);
    }

    #[test]
    fn test_ledger_migration() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_ledger.db");

        let ledger = LedgerManager::new(&db_path).unwrap();

        let entry = LedgerEntry {
            id: "test-id-123".to_string(),
            source_path: "/test/path.pdf".to_string(),
            checksum: "sha256:abc123".to_string(),
            pages_processed: 10,
            extraction_engine: "docling".to_string(),
            extraction_version: "2.68.0".to_string(),
            processing_time_ms: 1500,
            status: "success".to_string(),
            error_message: None,
            metadata_json: "{}".to_string(),
            created_at: Utc::now().into(),
        };

        let result = ledger.migrate_to_ledger(entry.clone());
        assert!(result.is_ok());

        // Verify retrieval
        let retrieved = ledger.get_by_checksum(&entry.checksum).unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, entry.id);
        assert_eq!(retrieved.checksum, entry.checksum);
        assert_eq!(retrieved.pages_processed, entry.pages_processed);
    }

    #[test]
    fn test_idempotent_migration() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_ledger.db");

        let ledger = LedgerManager::new(&db_path).unwrap();

        let entry = LedgerEntry {
            id: "test-id-456".to_string(),
            source_path: "/test/path2.pdf".to_string(),
            checksum: "sha256:def456".to_string(),
            pages_processed: 15,
            extraction_engine: "docling".to_string(),
            extraction_version: "2.68.0".to_string(),
            processing_time_ms: 2000,
            status: "success".to_string(),
            error_message: None,
            metadata_json: "{}".to_string(),
            created_at: Utc::now().into(),
        };

        // Migrate twice (idempotent)
        let result1 = ledger.migrate_to_ledger(entry.clone());
        assert!(result1.is_ok());

        let result2 = ledger.migrate_to_ledger(entry);
        assert!(result2.is_ok());

        // Should still be only one entry
        let stats = ledger.get_stats().unwrap();
        assert_eq!(stats.total_entries, 1);
    }
}
