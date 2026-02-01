/*
 * TACHFILETO ENTERPRISE CORE - V2.5 (FOUNDER'S EYE)
 * ==================================================
 * Copyright (c) 2025 TachFileTo Team. All rights reserved.
 * Build Date: December 26, 2025
 * Engine: Polars 0.52 + Calamine 0.32 + Rust 2021
 * Architecture: Iron Core (Deterministic Validation)
 */

mod dashboard;
mod excel_engine;
mod normalizer;
mod commands; 
pub mod core_contract;
pub mod resource_court;
pub mod executioner;
mod telemetry;
mod telemetry_state;

use excel_engine::ExcelAppState;
use iron_table::contract::{TableTruth, ProjectTruth};
use iron_adapter::diagnostics::{StructuredRejection, TruthSnapshot};
use std::sync::Mutex;

use crate::core_contract::ui_contract::{FileStatus, FileStatusLabel};

pub struct ForensicState {
    pub active_table: Mutex<Option<TableTruth>>,
    pub active_violations: Mutex<Vec<StructuredRejection>>,
    pub active_project_truth: Mutex<Option<ProjectTruth>>,
    pub ingested_files: Mutex<Vec<FileStatus>>,
    pub audit_ledger: Mutex<Vec<TruthSnapshot>>,
}

impl Default for ForensicState {
    fn default() -> Self {
        Self {
            active_table: Mutex::new(None),
            active_violations: Mutex::new(Vec::new()),
            active_project_truth: Mutex::new(None),
            ingested_files: Mutex::new(vec![
                FileStatus {
                    name: "Tri-Conflict-Pack.pdf".to_string(),
                    status: FileStatusLabel::Tainted,
                    timestamp: "2026-01-31 22:50".to_string(),
                    progress: Some(100.0),
                },
                FileStatus {
                    name: "BM_01_Kiem_Dinh.pdf".to_string(),
                    status: FileStatusLabel::Clean,
                    timestamp: "2026-01-31 20:00".to_string(),
                    progress: Some(100.0),
                },
            ]),
            audit_ledger: Mutex::new(Vec::new()),
        }
    }
}

use commands::validate_file;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(ExcelAppState::default())
        .manage(ForensicState::default())
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                telemetry::start_telemetry_loop(handle).await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            excel_engine::excel_load_file,
            excel_engine::excel_select_sheet,
            excel_engine::excel_get_window,
            excel_engine::excel_total_rows,
            dashboard::get_dashboard_summary,
            normalizer::cmd_normalize_descriptions,
            validate_file::validate_file,
            validate_file::extract_file,
            validate_file::get_ledger_entries,
            validate_file::generate_mock_ingestion,
            commands::repair::get_structural_diagnostics,
            commands::repair::apply_table_repairs,
            commands::repair::seal_table_truth,
            commands::repair::get_encoding_candidates,
            commands::repair::apply_table_repairs_to_active,
            commands::ui_bridge::get_file_ledger,
            commands::ui_bridge::get_table_truth,
            commands::ui_bridge::get_evidence,
            commands::ui_bridge::get_discrepancy,
            commands::ui_bridge::get_metric_lineage,
            commands::ui_bridge::get_audit_trail,
            commands::export::cmd_export_audit
        ])
        .run(tauri::generate_context!())
        .expect("Lỗi khi chạy Tauri application");
}
