//! TachFileTo - Antigravity PDF Processing Engine
//! Main entry point for Tauri v2 application

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod cache;
mod ipc;
mod strategy;

use ipc::{EvidenceCache, PythonWorker};
use std::sync::Arc;
use tokio::sync::Mutex as AsyncMutex;

fn main() {
    // Initialize logging
    env_logger::init();

    // Create shared state
    let worker = Arc::new(AsyncMutex::new(PythonWorker::new()));
    let cache = Arc::new(EvidenceCache::new(100));

    tauri::Builder::default()
        .manage(worker)
        .manage(cache)
        .invoke_handler(tauri::generate_handler![
            ipc::extract_evidence,
            ipc::get_evidence_health,
            ipc::clear_evidence_cache,
            ipc::restart_evidence_worker,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
