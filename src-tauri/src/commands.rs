// TachFileTo — Tauri IPC Commands
//
// RULE: These are thin adapters. Zero business logic here.
// All CPU-bound operations MUST use spawn_blocking (CTO requirement).
// RULE: MutexGuard MUST be dropped before any .await boundary.

use iron_engine::{DocumentSummary, IpcDiffReport, ProcessError};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;

// ─── Session Registry ─────────────────────────────────────────────────────────
pub struct DocumentRegistry(pub Mutex<HashMap<String, DocumentSummary>>);

// ─── Commands ─────────────────────────────────────────────────────────────────

/// Process a document file. Returns an opaque summary.
#[tauri::command]
pub async fn process_document(
    path: String,
    registry: State<'_, DocumentRegistry>,
) -> Result<DocumentSummary, ProcessError> {
    let path_buf = std::path::PathBuf::from(&path);

    let summary = tauri::async_runtime::spawn_blocking(move || {
        iron_engine::process_document(&path_buf)
    })
    .await
    .map_err(|_| ProcessError::EnginePanic)??;

    // Store in session registry — lock is scoped, dropped immediately
    {
        let mut reg = registry.0.lock().map_err(|_| ProcessError::EnginePanic)?;
        reg.insert(summary.id.clone(), summary.clone());
    } // MutexGuard dropped here — no await boundary crossed

    Ok(summary)
}

/// Export the cached Markdown for a processed document (by ID).
#[tauri::command]
pub async fn export_markdown(
    id: String,
    registry: State<'_, DocumentRegistry>,
) -> Result<String, ProcessError> {
    // Extract the markdown string before any await — drop lock immediately
    let md = {
        let reg = registry.0.lock().map_err(|_| ProcessError::EnginePanic)?;
        let summary = reg.get(&id).ok_or(ProcessError::IoError)?;
        iron_engine::get_markdown(summary).to_string()
    }; // MutexGuard dropped here

    Ok(md)
}

/// Compare two processed documents. Returns a diff report.
#[tauri::command]
pub async fn compare_documents(
    id_a: String,
    id_b: String,
    registry: State<'_, DocumentRegistry>,
) -> Result<IpcDiffReport, ProcessError> {
    // Clone both summaries before releasing the lock — no lock across .await
    let (a, b) = {
        let reg = registry.0.lock().map_err(|_| ProcessError::EnginePanic)?;
        let a = reg.get(&id_a).ok_or(ProcessError::IoError)?.clone();
        let b = reg.get(&id_b).ok_or(ProcessError::IoError)?.clone();
        (a, b)
    }; // MutexGuard dropped here — safe to .await below

    let report = tauri::async_runtime::spawn_blocking(move || {
        iron_engine::compare_documents(&a, &b)
    })
    .await
    .map_err(|_| ProcessError::EnginePanic)?;

    Ok(report)
}
