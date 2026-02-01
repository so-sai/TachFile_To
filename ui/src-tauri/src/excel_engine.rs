/*
 * TACHFILETO EXCEL ENGINE V3.0 - IRON CORE (12/2025)
 * ===========================================================
 * Enterprise-Grade Smart Header Detection
 * - Merged cell value propagation
 * - Fuzzy keyword matching (Jaro-Winkler)
 * - Multi-method header detection fallback
 * - Footer row skipping
 * - VND comma handling
 */

use anyhow::{anyhow, Result};
use calamine::{Data, Reader, open_workbook_auto};
use polars::prelude::*;
use serde::Serialize;
use std::sync::Mutex;
use std::collections::HashMap;
use tauri::{State, Emitter};

// Import Column Normalizer
use crate::normalizer::GLOBAL_NORMALIZER;
use crate::ForensicState;

// --- 1. APP STATE (TRUTH CONTRACT) ---
pub struct ExcelAppState {
    pub df: Mutex<Option<DataFrame>>,
}

impl Default for ExcelAppState {
    fn default() -> Self {
        Self {
            df: Mutex::new(None),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ExcelWindow {
    pub columns: Vec<String>,
    pub data: Vec<Vec<String>>,
    pub total_rows: usize,
}

/// V3.1: Response struct with sheet list for Sheet Selector feature
#[derive(Debug, Serialize)]
pub struct ExcelLoadResponse {
    pub columns: Vec<String>,
    pub data: Vec<Vec<String>>,
    pub total_rows: usize,
    pub sheets: Vec<String>,
    pub current_sheet: String,
}

/// Propagate merged cell values (fill empty cells from left/above)
fn propagate_merged_values(rows: &mut Vec<Vec<String>>) {
    // Simple horizontal propagation: if cell is empty, copy from left
    for row in rows.iter_mut() {
        let mut last_value = String::new();
        for cell in row.iter_mut() {
            if cell.trim().is_empty() && !last_value.is_empty() {
                // Propagate value from previous cell
                *cell = last_value.clone();
            } else {
                last_value = cell.clone();
            }
        }
    }
    
    // Vertical propagation for header-purposes (not usually needed for data, but can help metadata)
    let header_scan_limit = std::cmp::min(10, rows.len());
    for col_idx in 0..rows.get(0).map(|r| r.len()).unwrap_or(0) {
        let mut last_value = String::new();
        for row_idx in 0..header_scan_limit {
            if let Some(row) = rows.get_mut(row_idx) {
                if let Some(cell) = row.get_mut(col_idx) {
                    if cell.trim().is_empty() && !last_value.is_empty() {
                        *cell = last_value.clone();
                    } else {
                        last_value = cell.clone();
                    }
                }
            }
        }
    }
}

/// Clean column name: remove annotations, newlines, extra spaces
fn clean_column_name(name: &str) -> String {
    name.trim()
        .replace("\n", " ")
        .replace("\r", "")
        .replace("  ", " ")
        // Vietnamese currency annotations
        .replace("(VNĐ)", "")
        .replace("(vnđ)", "")
        .replace("(VND)", "")
        .replace("(vnd)", "")
        .replace("(đồng)", "")
        .replace("(Đồng)", "")
        .replace("(nghìn đồng)", "")
        .replace("(triệu đồng)", "")
        .replace("(tỷ đồng)", "")
        .replace("(đ)", "")
        // Common unit suffixes
        .replace("(m2)", "")
        .replace("(m3)", "")
        .replace("(kg)", "")
        .replace("(tấn)", "")
        .trim()
        .to_string()
}

/// Fuzzy keyword matching using Jaro-Winkler similarity
fn fuzzy_contains(text: &str, keyword: &str) -> bool {
    let text_lower = text.to_lowercase();
    let keyword_lower = keyword.to_lowercase();
    
    // Exact substring match first
    if text_lower.contains(&keyword_lower) {
        return true;
    }
    
    // Fuzzy match for Vietnamese variations (diacritics)
    let similarity = strsim::jaro_winkler(&text_lower, &keyword_lower);
    similarity > 0.85
}

/// Calculate keyword score for a row
fn calculate_keyword_score(row: &[String]) -> (u32, Vec<String>) {
    const QS_KEYWORDS: &[&str] = &[
        "stt", "tt", "hạng mục", "hang muc", "tên", "ten", "mô tả", "mo ta",
        "diễn giải", "dien giai", "đơn vị", "don vi", "dvt", "khối lượng", 
        "khoi luong", "kl", "đơn giá", "don gia", "đg", "thành tiền", "thanh tien",
        "ghi chú", "ghi chu", "vật tư", "vat tu", "nhân công", "nhan cong",
        "máy", "may", "định mức", "dinh muc", "mã số", "ma so", "mã hiệu", "ma hieu"
    ];
    
    let mut score = 0;
    let mut found_keywords = Vec::new();
    
    for cell in row {
        let cell_lower = cell.to_lowercase();
        for keyword in QS_KEYWORDS {
            if fuzzy_contains(&cell_lower, keyword) {
                score += 1;
                found_keywords.push(keyword.to_string());
                break;
            }
        }
    }
    
    (score, found_keywords)
}

/// Calculate numeric score (rows with numbers are likely data, not headers)
fn calculate_numeric_score(row: &[String]) -> u32 {
    row.iter()
        .filter(|cell| {
            let cleaned = cell.replace(",", "").replace(".", "");
            cleaned.parse::<f64>().is_ok() && cleaned.len() > 2
        })
        .count() as u32
}

/// Smart header detection with multiple methods
fn smart_detect_header(rows: &[Vec<String>]) -> (usize, u32, Vec<String>) {
    let scan_limit = std::cmp::min(50, rows.len());
    
    let mut best_row_idx = 0;
    let mut best_keyword_score = 0;
    let mut best_keywords = Vec::new();
    
    // Method 1: Keyword matching (primary)
    for (row_idx, row) in rows.iter().take(scan_limit).enumerate() {
        let (score, keywords) = calculate_keyword_score(row);
        
        // Penalize rows with too many numbers (likely data rows)
        let numeric_penalty = calculate_numeric_score(row);
        let adjusted_score = if numeric_penalty > 3 { score.saturating_sub(2) } else { score };
        
        if adjusted_score > best_keyword_score {
            best_keyword_score = adjusted_score;
            best_row_idx = row_idx;
            best_keywords = keywords;
        }
    }
    
    // Method 2: If keyword score is low, try pattern matching
    if best_keyword_score < 3 {
        for (row_idx, row) in rows.iter().take(scan_limit).enumerate() {
            let has_stt_pattern = row.iter().any(|c| {
                let lower = c.to_lowercase();
                lower.contains("stt") || lower.contains("số tt") || lower == "tt"
            });
            let has_name_pattern = row.iter().any(|c| {
                let lower = c.to_lowercase();
                lower.contains("tên") || lower.contains("ten") || lower.contains("hạng mục")
            });
            
            if has_stt_pattern && has_name_pattern {
                println!("   → Pattern match fallback: row {}", row_idx + 1);
                return (row_idx, 2, vec!["pattern_match".to_string()]);
            }
        }
    }
    
    (best_row_idx, best_keyword_score, best_keywords)
}

/// Check if a row is a footer row (contains "Tổng cộng", etc.)
fn is_footer_row(row: &[String]) -> bool {
    const FOOTER_KEYWORDS: &[&str] = &[
        "tổng cộng", "tong cong", "cộng", "tổng:", "tong:", 
        "total", "grand total", "subtotal", "cộng phát sinh"
    ];
    
    row.iter().any(|cell| {
        let lower = cell.to_lowercase();
        FOOTER_KEYWORDS.iter().any(|kw| lower.contains(kw))
    })
}

/// V3.1: Find best sheet based on Vietnamese QS keywords
fn find_best_sheet(sheet_names: &[String]) -> String {
    const TARGET_KEYWORDS: &[&str] = &[
        "khối lượng", "khoi luong", "dự toán", "du toan", "báo giá", "bao gia", "boq", "tổng hợp"
    ];
    const EXCLUDE_KEYWORDS: &[&str] = &[
        "bìa", "bia", "ghi chú", "ghi chu", "thông tin", "thong tin", "cover", "note"
    ];

    // Priority 1: Target sheets
    for sheet in sheet_names {
        let lower = sheet.to_lowercase();
        if TARGET_KEYWORDS.iter().any(|kw| lower.contains(kw)) {
            println!("   ✔️ Sheet Selector: Found target sheet '{}'", sheet);
            return sheet.clone();
        }
    }

    // Priority 2: Non-excluded sheets (first one)
    for sheet in sheet_names {
        let lower = sheet.to_lowercase();
        if !EXCLUDE_KEYWORDS.iter().any(|kw| lower.contains(kw)) {
            println!("   ⚠️ Sheet Selector: Using fallback sheet '{}'", sheet);
            return sheet.clone();
        }
    }

    // Fallback: First sheet
    sheet_names.first().cloned().unwrap_or_else(|| "Sheet1".to_string())
}

// --- 2. CORE ENGINE (PURE RUST) ---

/// Đọc file Excel thô (Universal Support: .xls, .xlsx, .xlsb)
pub fn read_raw_excel(
    file_path: &str, 
    sheet_name: Option<&str>,
    app_handle: Option<&tauri::AppHandle>
) -> Result<DataFrame> {
    let file_name = std::path::Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(file_path)
        .to_string();

    println!("\n🚀 IRON CORE V3.0 - EXCEL ENGINE STARTING");
    println!("   → File: {}", file_path);
    
    // 1. Auto-detect file format and open workbook
    let mut workbook = open_workbook_auto(file_path)
        .map_err(|e| anyhow!("Không thể đọc file Excel (có thể do sai định dạng hoặc file đang mở): {}", e))?;

    // 2. Get sheet (first sheet or by name)
    let range = match sheet_name {
        Some(name) => workbook
            .worksheet_range(name)
            .map_err(|e| anyhow!("Lỗi đọc sheet '{}': {}", name, e))?,
        None => workbook
            .worksheet_range_at(0)
            .ok_or_else(|| anyhow!("File Excel không có sheet nào"))?
            .map_err(|e| anyhow!("Lỗi đọc sheet đầu tiên: {}", e))?,
    };

    // 3. Convert to String matrix (handle all data types)
    let total_raw_rows = range.rows().count();
    let mut rows: Vec<Vec<String>> = Vec::with_capacity(total_raw_rows);
    let mut last_progress_emit = std::time::Instant::now();
    let mut current_row_count = 0;

    for row in range.rows() {
        current_row_count += 1;
        
        // Convert row to String
        let row_vec: Vec<String> = row.iter()
            .map(|cell| match cell {
                Data::Empty => String::new(),
                Data::String(s) => s.trim().to_string(),
                Data::Float(f) => f.to_string(),
                Data::Int(i) => i.to_string(),
                Data::Bool(b) => b.to_string(),
                Data::DateTime(dt) => dt.to_string(),
                Data::Error(e) => format!("Lỗi_{:?}", e),
                _ => String::new(),
            })
            .collect();
        rows.push(row_vec);

        // 🛡️ PROGRESS EMISSION WITH THROTTLING
        if let Some(handle) = app_handle {
            // Only emit every 2% or every 200ms to avoid IPC flood
            let progress = (current_row_count as f64 / total_raw_rows as f64) * 100.0;
            if last_progress_emit.elapsed().as_millis() > 200 || current_row_count == total_raw_rows {
                let _ = handle.emit("file-progress", serde_json::json!({
                    "fileName": file_name,
                    "progress": progress
                }));
                last_progress_emit = std::time::Instant::now();
            }
        }
    }

    if rows.is_empty() {
        return Err(anyhow!("File Excel rỗng"));
    }
    
    println!("   → Raw rows loaded: {}", rows.len());

    // 4. Propagate merged cell values
    propagate_merged_values(&mut rows);

// ============================================================
    // SMART HEADER DETECTION V3.0 - ENTERPRISE GRADE
    // ============================================================
    let (header_row_index, keyword_score, detected_keywords) = smart_detect_header(&rows);
    
    println!("🎯 Smart Header Detection V3.0:");
    println!("   → Best header row: {} (score: {})", header_row_index + 1, keyword_score);
    println!("   → Keywords found: {:?}", detected_keywords);
    
    // Fallback mechanism
    let final_header_row = if keyword_score >= 3 {
        println!("   ✅ High confidence header detected");
        header_row_index
    } else if keyword_score > 0 {
        println!("   ⚠️  Low confidence - using best guess (row {})", header_row_index + 1);
        header_row_index
    } else {
        println!("   ❌ No QS keywords found - fallback to row 1");
        0
    };
    
    let raw_headers = &rows[final_header_row];
    
    // 5. Filter out footer rows
    let data_rows: Vec<&Vec<String>> = rows[final_header_row + 1..]
        .iter()
        .filter(|row| !is_footer_row(row))
        .collect();
    
    println!("   → Data rows after footer filter: {}", data_rows.len());

    // 6. --- FIX: LẤP ĐẦY HEADER TRỐNG (MERGED CELLS) ---
    let mut filled_headers = Vec::new();
    let mut last_valid_header = "Column".to_string();

    for h in raw_headers {
        let current = clean_column_name(h);
        if !current.is_empty() {
            last_valid_header = current;
            filled_headers.push(last_valid_header.clone());
        } else {
            // Nếu rỗng, dùng tên cột trước đó + hậu tố (VD: DonGia_sub)
            filled_headers.push(format!("{}_sub", last_valid_header));
        }
    }

    // Xử lý trùng tên (Deduplicate)
    let mut final_headers = Vec::new();
    let mut name_counts: HashMap<String, usize> = HashMap::new();
    for h in filled_headers {
        let count = name_counts.entry(h.clone()).or_insert(0);
        *count += 1;
        final_headers.push(if *count == 1 { h } else { format!("{}_{}", h, count) });
    }
    
    println!("   → Final headers: {:?}", &final_headers[..std::cmp::min(5, final_headers.len())]);

    // 7. Build Polars DataFrame
    let mut series_vec: Vec<Column> = Vec::new();

    for (col_idx, header) in final_headers.iter().enumerate() {
        let column_data: Vec<String> = data_rows
            .iter()
            .map(|row| row.get(col_idx).cloned().unwrap_or_default())
            .collect();

        // Smart numeric parsing (handle VND comma format: 1,000,000)
        let numeric_values: Vec<Option<f64>> = column_data
            .iter()
            .map(|s| s.replace(",", "").replace(" ", "").parse::<f64>().ok())
            .collect();

        let all_numeric = numeric_values.iter().all(|v| v.is_some()) && !column_data.is_empty();

        let series = if all_numeric {
            let nums: Vec<f64> = numeric_values.into_iter().flatten().collect();
            Series::new(header.into(), nums)
        } else {
            Series::new(header.into(), column_data)
        };

        series_vec.push(series.into());
    }

    let df = DataFrame::new(series_vec).map_err(|e| anyhow!("Lỗi tạo DataFrame: {}", e))?;
    
    println!("   ✅ DataFrame created: {} rows x {} columns", df.height(), df.width());
    println!("🏁 IRON CORE V3.0 - PROCESSING COMPLETE\n");
    
    Ok(df)
}

// --- 3. TAURI COMMANDS (INTERFACE VỚI REACT) ---

#[tauri::command]
pub async fn excel_load_file(
    path: String,
    app_handle: tauri::AppHandle,
    excel_state: State<'_, ExcelAppState>,
    forensic_state: State<'_, ForensicState>,
) -> Result<ExcelLoadResponse, String> {
    let file_name = std::path::Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&path)
        .to_string();

    // 0. Handle PDF Ingestion (Bridge for V1.0.1)
    if path.to_lowercase().ends_with(".pdf") {
        println!("[Mission 037] PDF Ingestion Detected: {}", path);
        
        let mut ingested_guard = forensic_state.ingested_files.lock().map_err(|_| "Lỗi lock ingested_files")?;
        if !ingested_guard.iter().any(|f| f.name == file_name) {
            ingested_guard.push(crate::core_contract::ui_contract::FileStatus {
                name: file_name.clone(),
                status: crate::core_contract::ui_contract::FileStatusLabel::Tainted, // Temporary status
                timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
                progress: Some(0.0), // Start with 0%
            });
        }

        // Return a mock response for UI stability
        return Ok(ExcelLoadResponse {
            columns: vec!["ID".into(), "Nội dung".into(), "Trạng thái".into()],
            data: vec![vec!["1".into(), format!("Hồ sơ PDF: {}", file_name), "ĐANG CHỜ CHIẾT XUẤT".into()]],
            total_rows: 1,
            sheets: vec!["PDF_VIEW".into()],
            current_sheet: "PDF_VIEW".into(),
        });
    }

    // 1. Mở workbook để lấy danh sách sheet
    let workbook = open_workbook_auto(&path).map_err(|e| e.to_string())?;
    let sheet_names: Vec<String> = workbook.sheet_names().to_owned();
    
    // 2. Chọn sheet tốt nhất tự động
    let target_sheet = find_best_sheet(&sheet_names);
    
    // 3. Đọc và parse Excel từ sheet đã chọn
    let mut df = read_raw_excel(&path, Some(&target_sheet), Some(&app_handle)).map_err(|e| e.to_string())?;

    // 4. Tự động chuẩn hóa tên cột (QS Standards)
    let _ = GLOBAL_NORMALIZER.normalize_dataframe_columns(&mut df);

    // 5. Metadata cho window ban đầu (100 dòng)
    let headers: Vec<String> = df.get_column_names().iter().map(|s| (*s).to_string()).collect();
    let total_rows = df.height();
    
    let window_size = 100.min(total_rows);
    let mut window_data = Vec::new();

    // Convert sang String cho UI
    for i in 0..window_size {
        let mut row = Vec::new();
        for col in &headers {
            let val = df.column(col).and_then(|c| c.get(i)).map(|v| format!("{}", v)).unwrap_or_default();
            row.push(val);
        }
        window_data.push(row);
    }

    // 6. Lưu vào RAM
    let mut excel_state_guard = excel_state.df.lock().map_err(|_| "Lỗi lock excel_state".to_string())?;
    *excel_state_guard = Some(df);

    // 7. Cập nhật Sổ cái (Ledger)
    {
        let mut ingested_guard = forensic_state.ingested_files.lock().map_err(|_| "Lỗi lock ingested_files")?;
        if !ingested_guard.iter().any(|f| f.name == file_name) {
            ingested_guard.push(crate::core_contract::ui_contract::FileStatus {
                name: file_name.clone(),
                status: crate::core_contract::ui_contract::FileStatusLabel::Clean,
                timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
                progress: Some(100.0),
            });
        }
    }

    Ok(ExcelLoadResponse {
        columns: headers,
        data: window_data,
        total_rows,
        sheets: sheet_names,
        current_sheet: target_sheet,
    })
}

/// V3.1: Người dùng tự chọn sheet khác
#[tauri::command]
pub async fn excel_select_sheet(
    path: String,
    sheet_name: String,
    app_handle: tauri::AppHandle,
    state: State<'_, ExcelAppState>,
) -> Result<ExcelLoadResponse, String> {
    println!("[V3.1 Sheet Selector] User selected sheet: '{}'", sheet_name);
    
    // 1. Mở workbook để lấy danh sách sheet (cho response)
    let workbook = open_workbook_auto(&path).map_err(|e| e.to_string())?;
    let sheet_names: Vec<String> = workbook.sheet_names().to_owned();
    
    // 2. Đọc đích danh sheet người dùng chọn
    let mut df = read_raw_excel(&path, Some(&sheet_name), Some(&app_handle)).map_err(|e| e.to_string())?;

    println!(
        "[Iron Core V3.1] Switched to sheet '{}': {} rows x {} columns",
        sheet_name,
        df.height(),
        df.width()
    );

    // 3. Tự động chuẩn hóa tên cột
    let _ = GLOBAL_NORMALIZER.normalize_dataframe_columns(&mut df);

    // 4. Metadata cho window
    let headers: Vec<String> = df.get_column_names().iter().map(|s| (*s).to_string()).collect();
    let total_rows = df.height();
    
    let window_size = 100.min(total_rows);
    let mut window_data = Vec::new();

    for i in 0..window_size {
        let mut row = Vec::new();
        for col in &headers {
            let val = df.column(col).and_then(|c| c.get(i)).map(|v| format!("{}", v)).unwrap_or_default();
            row.push(val);
        }
        window_data.push(row);
    }

    // 5. Lưu vào RAM
    let mut state_guard = state.df.lock().map_err(|_| "Lỗi lock state".to_string())?;
    *state_guard = Some(df);

    Ok(ExcelLoadResponse {
        columns: headers,
        data: window_data,
        total_rows,
        sheets: sheet_names,
        current_sheet: sheet_name,
    })
}

#[tauri::command]
pub async fn excel_get_window(
    start: usize,
    end: usize,
    state: State<'_, ExcelAppState>,
) -> Result<ExcelWindow, String> {
    let state_guard = state.df.lock().map_err(|_| "Lỗi lock state".to_string())?;

    let df = match state_guard.as_ref() {
        Some(df) => df,
        None => return Err("Chưa có dữ liệu".to_string()),
    };

    let total_rows = df.height();
    let safe_end = end.min(total_rows);
    
    if start >= safe_end {
        return Ok(ExcelWindow {
            columns: Vec::new(),
            data: Vec::new(),
            total_rows,
        });
    }

    let len = safe_end - start;
    let sliced_df = df.slice(start as i64, len);
    let columns: Vec<String> = sliced_df.get_column_names().iter().map(|s| (*s).to_string()).collect();

    let mut data = Vec::new();
    for i in 0..sliced_df.height() {
        let mut row = Vec::new();
        for col in &columns {
            let val = sliced_df.column(col).and_then(|c| c.get(i)).map(|v| format!("{}", v)).unwrap_or_default();
            row.push(val);
        }
        data.push(row);
    }

    Ok(ExcelWindow {
        columns,
        data,
        total_rows,
    })
}

#[tauri::command]
pub async fn excel_total_rows(state: State<'_, ExcelAppState>) -> Result<usize, String> {
    let state_guard = state.df.lock().map_err(|_| "Lỗi lock state".to_string())?;
    Ok(state_guard.as_ref().map(|df| df.height()).unwrap_or(0))
}
