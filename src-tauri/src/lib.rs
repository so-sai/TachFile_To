// TachFileTo — Tauri Shell Entry Point
// No ghost terminal window in production build (elite-ui-rust skill compliance)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // In dev mode, open DevTools automatically
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::process_document,
            commands::export_markdown,
            commands::compare_documents,
        ])
        .run(tauri::generate_context!())
        .expect("Lỗi khởi động TachFileTo");
}
