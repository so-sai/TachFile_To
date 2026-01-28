use rusqlite::{params, Connection, Result};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub struct IngestionRecord {
    pub file_hash: String,
    pub file_path: String,
    pub page_count: i32,
    pub status: String,
    pub processed_at: String,
}

const DB_PATH: &str = "ingestion.db";

pub fn init_db() -> Result<()> {
    let conn = Connection::open(DB_PATH)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS pdf_ingestion (
            file_hash TEXT PRIMARY KEY,
            file_path TEXT NOT NULL,
            page_count INTEGER NOT NULL,
            status TEXT NOT NULL,
            processed_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    Ok(())
}

pub fn hash_file<P: AsRef<Path>>(path: P) -> std::io::Result<String> {
    let file = File::open(path.as_ref())?;
    // Use 64KB buffer to reduce syscalls as per user suggestion
    let mut reader = BufReader::with_capacity(64 * 1024, file);
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    let hash = hex::encode(hasher.finalize());
    Ok(hash)
}

pub fn check_file_processed(hash: &str) -> Result<Option<IngestionRecord>> {
    let conn = Connection::open(DB_PATH)?;
    let mut stmt = conn.prepare(
        "SELECT file_hash, file_path, page_count, status, processed_at 
         FROM pdf_ingestion WHERE file_hash = ?",
    )?;

    let mut rows = stmt.query(params![hash])?;

    if let Some(row) = rows.next()? {
        Ok(Some(IngestionRecord {
            file_hash: row.get(0)?,
            file_path: row.get(1)?,
            page_count: row.get(2)?,
            status: row.get(3)?,
            processed_at: row.get(4)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn record_ingestion(path: &str, hash: &str, pages: i32, status: &str) -> Result<()> {
    let conn = Connection::open(DB_PATH)?;
    conn.execute(
        "INSERT OR REPLACE INTO pdf_ingestion (file_hash, file_path, page_count, status)
         VALUES (?, ?, ?, ?)",
        params![hash, path, pages, status],
    )?;
    Ok(())
}
