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
use commands::validate_file;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(ExcelAppState::default())
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
            validate_file::generate_mock_ingestion
        ])
        .run(tauri::generate_context!())
        .expect("Lỗi khi chạy Tauri application");
}
