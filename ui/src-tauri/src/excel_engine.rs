/*
 * TACHFILETO EXCEL ENGINE V2.5 - POLARS 0.52 + CALAMINE 0.32
 * ===========================================================
 * FIX: Đồng bộ API Polars 0.52 (.to_owned().into_series() / Series -> Column)
 * FIX: Handle Calamine Data::DateTime
 */

use calamine::{Data, Reader, Xlsx, open_workbook};
use polars::prelude::*;
use serde::Serialize;
use std::sync::Mutex;

// Import Column Normalizer
use crate::normalizer::GLOBAL_NORMALIZER;

// AppState để chia sẻ DataFrame giữa các command
#[derive(Default)]
pub struct ExcelAppState {
    pub df: Mutex<Option<DataFrame>>,
}

impl ExcelAppState {
    pub fn new() -> Self {
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

#[tauri::command]
pub async fn excel_load_file(
    path: String,
    state: tauri::State<'_, ExcelAppState>,
) -> Result<ExcelWindow, String> {
    // 1. Đọc file Excel bằng Calamine
    let mut workbook: Xlsx<_> =
        open_workbook(&path).map_err(|e| format!("Lỗi mở file Excel: {}", e))?;

    let range = workbook
        .worksheet_range_at(0)
        .ok_or("File không có sheet nào")?
        .map_err(|e| format!("Lỗi đọc sheet: {}", e))?;

    // 2. Convert thành Polars DataFrame
    let mut headers = Vec::new();
    let mut data_rows = Vec::new();

    for (row_idx, row) in range.rows().enumerate() {
        if row_idx == 0 {
            // Header row
            headers = row
                .iter()
                .enumerate()
                .map(|(i, cell)| match cell {
                    Data::Empty => format!("Cột_{}", i + 1),
                    Data::String(s) => s.trim().to_string(),
                    Data::Float(f) => f.to_string(),
                    Data::Int(i) => i.to_string(),
                    Data::Bool(b) => b.to_string(),
                    Data::DateTime(dt) => dt.to_string(),
                    Data::Error(e) => format!("Lỗi_{:?}", e),
                    _ => format!("Cột_{}", i + 1),
                })
                .collect();
        } else {
            // Data rows
            let row_data: Vec<String> = row
                .iter()
                .map(|cell| match cell {
                    Data::Empty => String::new(),
                    Data::String(s) => s.trim().to_string(),
                    Data::Float(f) => f.to_string(),
                    Data::Int(i) => i.to_string(),
                    Data::Bool(b) => b.to_string(),
                    Data::DateTime(dt) => dt.to_string(),
                    Data::Error(e) => format!("{:?}", e),
                    _ => String::new(),
                })
                .collect();
            data_rows.push(row_data);
        }
    }

    // 3. Tạo Polars DataFrame
    // FIX POLARS 0.52: Mismatched types (Vec<Series> vs Vec<Column>)
    let mut series_vec: Vec<Column> = Vec::new();

    for (col_idx, header) in headers.iter().enumerate() {
        let column_data: Vec<String> = data_rows
            .iter()
            .map(|row| row.get(col_idx).cloned().unwrap_or_default())
            .collect();

        // Thử parse thành số nếu có thể
        let numeric_values: Vec<Option<f64>> =
            column_data.iter().map(|s| s.parse::<f64>().ok()).collect();

        let all_numeric = numeric_values.iter().all(|v| v.is_some());

        let series = if all_numeric && !column_data.is_empty() {
            // Tất cả đều là số
            let nums: Vec<f64> = numeric_values.into_iter().flatten().collect();
            Series::new(header.into(), nums)
        } else {
            // Giữ nguyên dạng string
            Series::new(header.into(), column_data)
        };

        // FIX: Chuyển Series -> Column
        series_vec.push(series.into());
    }

    let mut df = DataFrame::new(series_vec).map_err(|e| format!("Lỗi tạo DataFrame: {}", e))?;

    println!(
        "[ExcelEngine] Loaded {} rows, {} columns",
        df.height(),
        df.width()
    );

    // 🎯 ÁP DỤNG COLUMN NORMALIZER - Chuẩn hóa tên cột tiếng Việt
    println!("[ExcelEngine] Applying Column Normalizer...");
    let normalization_results = GLOBAL_NORMALIZER
        .normalize_dataframe_columns(&mut df)
        .map_err(|e| format!("Lỗi chuẩn hóa cột: {}", e))?;

    // Log kết quả chuẩn hóa
    for result in &normalization_results {
        if result.original_name != result.normalized_name {
            println!(
                "  • '{}' → '{}' ({:?})",
                result.original_name, result.normalized_name, result.column_type
            );
        }
    }

    // 4. Lưu vào state
    let mut state_guard = state.df.lock().map_err(|_| "Lỗi lock state".to_string())?;
    *state_guard = Some(df.clone());

    // 5. Trả về window data (chỉ 100 dòng đầu)
    let window_size = 100;
    let limited_data: Vec<Vec<String>> = data_rows.iter().take(window_size).cloned().collect();

    Ok(ExcelWindow {
        columns: headers,
        data: limited_data,
        total_rows: data_rows.len(),
    })
}

#[tauri::command]
pub async fn excel_get_window(
    start: usize,
    end: usize,
    state: tauri::State<'_, ExcelAppState>,
) -> Result<ExcelWindow, String> {
    let state_guard = state.df.lock().map_err(|_| "Lỗi lock state".to_string())?;

    let df = match state_guard.as_ref() {
        Some(df) => df.clone(),
        None => return Err("Chưa có dữ liệu".to_string()),
    };

    let total_rows = df.height();
    let end = end.min(total_rows);

    if start >= end {
        return Ok(ExcelWindow {
            columns: Vec::new(),
            data: Vec::new(),
            total_rows: 0,
        });
    }

    // Lấy slice của DataFrame
    let sliced_df = df.slice(start as i64, end - start);

    // Convert sang format cho frontend
    let columns: Vec<String> = sliced_df
        .get_column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();

    let mut data = Vec::new();
    for i in 0..sliced_df.height() {
        let mut row = Vec::new();
        for col in &columns {
            let value = sliced_df
                .column(col)
                .and_then(|c| c.get(i))
                .map(|v| format!("{}", v))
                .unwrap_or_default();
            row.push(value);
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
pub async fn excel_total_rows(state: tauri::State<'_, ExcelAppState>) -> Result<usize, String> {
    let state_guard = state.df.lock().map_err(|_| "Lỗi lock state".to_string())?;

    match state_guard.as_ref() {
        Some(df) => Ok(df.height()),
        None => Ok(0),
    }
}
