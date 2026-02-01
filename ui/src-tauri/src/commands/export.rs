//! Export Commands: GUI → Desktop Files
//! 
//! **Mission 031: The Final Export**
//! - Integrates iron_engine::exporter with Tauri
//! - Handles Windows "File Lock" PermissionDenied errors

use tauri::{State, AppHandle};
use tauri_plugin_dialog::DialogExt;
use std::fs::File;
use std::io::Write;
use crate::ForensicState;
use crate::excel_engine::ExcelAppState;
use iron_engine::exporter::{MarkdownExporter, ExcelExporter};

#[tauri::command]
pub async fn cmd_export_audit(
    app: AppHandle,
    format: String, // "md" or "xlsx"
    forensic: State<'_, ForensicState>,
    excel_state: State<'_, ExcelAppState>,
) -> Result<String, String> {
    // 1. Get Truth Data
    let project_truth = {
        let guard = forensic.active_project_truth.lock().map_err(|_| "Lỗi lock forensic".to_string())?;
        guard.clone().ok_or_else(|| "Không có dữ liệu Project Truth để xuất.".to_string())?
    };

    let df = {
        let guard = excel_state.df.lock().map_err(|_| "Lỗi lock excel state".to_string())?;
        guard.clone().ok_or_else(|| "Không có dữ liệu bảng để xuất.".to_string())?
    };

    let table_meta = {
        let guard = forensic.active_table.lock().map_err(|_| "Lỗi lock table state".to_string())?;
        guard.clone().ok_or_else(|| "Không có metadata bảng.".to_string())?
    };

    // 2. Select file path via Dialog
    let extension = if format == "md" { "md" } else { "xlsx" };
    let file_path = app.dialog()
        .file()
        .add_filter("Audit Files", &[extension])
        .set_title("Lưu biên bản giám định")
        .set_file_name(&format!("TachFileTo_Export_{}.{}", project_truth.project_name, extension))
        .blocking_save_file();

    let path = match file_path {
        Some(p) => p.to_string(),
        None => return Ok("Hủy bỏ xuất file".to_string()),
    };

    // 3. Generate Content
    let result = if format == "md" {
        let content = MarkdownExporter::export(&project_truth);
        write_file_with_lock_check(&path, content.as_bytes())
    } else {
        let buffer = ExcelExporter::export(&df, &table_meta)
            .map_err(|e| format!("Lỗi tạo Excel: {:?}", e))?;
        write_file_with_lock_check(&path, &buffer)
    };

    match result {
        Ok(_) => Ok(format!("Đã xuất file thành công: {}", path)),
        Err(e) => Err(e),
    }
}

/// Helper: Write file with Windows PermissionDenied (Error 32) check
fn write_file_with_lock_check(path: &str, content: &[u8]) -> Result<(), String> {
    match File::create(path) {
        Ok(mut file) => {
            file.write_all(content).map_err(|e| format!("Lỗi khi ghi file: {}", e))?;
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied || e.raw_os_error() == Some(32) => {
            Err("Không thể ghi file. Vui lòng đóng file đang mở (Excel/Word) trước khi xuất đè.".to_string())
        }
        Err(e) => Err(format!("Lỗi truy cập file ({}): {}", e.raw_os_error().unwrap_or(0), e)),
    }
}
