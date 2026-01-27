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
mod commands; // NEW: Mission 2026-001
pub mod core_contract; // NEW: iron_core Embassy (Mission 2026-001)

use excel_engine::ExcelAppState;
use commands::validate_file; // Import new module

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(ExcelAppState::default())
        .invoke_handler(tauri::generate_handler![
            // Legacy V2.5 Commands
            excel_engine::excel_load_file,
            excel_engine::excel_select_sheet,
            excel_engine::excel_get_window,
            excel_engine::excel_total_rows,
            dashboard::get_dashboard_summary,
            normalizer::cmd_normalize_descriptions,
            // New Mission 2026-001 Commands
            validate_file::validate_file,
            validate_file::extract_file,
            validate_file::generate_mock_ingestion
        ])

        .run(tauri::generate_context!())
        .expect("Lỗi khi chạy Tauri application");
}
