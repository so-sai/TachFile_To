use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditEntry {
    pub timestamp: String,
    pub event_type: String, // "REJECTION", "REPAIR", "ASSEMBLY"
    pub table_id: String,
    pub details: serde_json::Value,
}

pub struct AuditLogger;

impl AuditLogger {
    /// Logs an entry to the audit file (JSON-L format).
    pub fn log(event_type: &str, table_id: &str, details: serde_json::Value, log_path: &Path) -> std::io::Result<()> {
        let entry = AuditEntry {
            timestamp: Utc::now().to_rfc3339(),
            event_type: event_type.to_uppercase(),
            table_id: table_id.to_string(),
            details,
        };

        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;
        
        let line = serde_json::to_string(&entry)? + "\n";
        file.write_all(line.as_bytes())?;
        Ok(())
    }
}
