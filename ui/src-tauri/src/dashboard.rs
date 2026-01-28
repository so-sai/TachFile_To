/*
 * TACHFILETO DASHBOARD MODULE V2.5
 * =================================
 * FIX: Polars 0.52 API (.to_owned().into_series() pattern)
 * FIX: Proper Column -> Series -> Sum conversion
 */

use polars::prelude::*;
use serde::Serialize;
use tauri::State;

use crate::excel_engine::ExcelAppState;

#[derive(Debug, Serialize, Clone)]
pub struct DashboardSummary {
    pub status: String,
    pub status_reason: String,
    pub top_risks: Vec<RiskItem>,
    pub payment_progress: PaymentProgress,
    pub pending_actions: Vec<String>,
    pub metrics: DashboardMetrics,
}

#[derive(Debug, Serialize, Clone)]
pub struct RiskItem {
    pub description: String,
    pub deviation: f64,
    pub impact: String,
    pub severity: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct PaymentProgress {
    pub received: f64,
    pub total_contract: f64,
    pub percent: f64,
    pub projected_profit: f64,
    pub profit_percent: f64,
}

#[derive(Debug, Serialize, Clone)]
pub struct DashboardMetrics {
    pub total_rows: usize,
    pub total_amount: f64,
    pub avg_deviation: f64,
    pub high_risk_count: usize,
    pub critical_count: usize,
    pub profit_margin_percent: f64,
    pub last_updated: String,
}

#[tauri::command]
pub async fn get_dashboard_summary(
    state: State<'_, ExcelAppState>,
) -> Result<DashboardSummary, String> {
    let state_guard = state.df.lock().map_err(|_| "Lỗi lock state".to_string())?;

    let df = match state_guard.as_ref() {
        Some(df) => df.clone(),
        None => return Err("Chưa có dữ liệu. Vui lòng tải file Excel trước.".to_string()),
    };

    // 🎯 1. PHÁT HIỆN CỘT TỰ ĐỘNG
    let column_names: Vec<String> = df
        .get_column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();

    println!("[Dashboard] Columns detected: {:?}", column_names);

    let (amount_col, calculated_col, measured_col, status_col) = detect_columns(&column_names);

    // 🎯 2. TÍNH TOÁN METRICS CƠ BẢN
    let total_rows = df.height();

    // Tính tổng tiền - FIX API POLARS 0.52
    let total_amount = if !amount_col.is_empty() {
        sum_column(&df, &amount_col)
    } else {
        0.0
    };

    // 🎯 3. TÍNH ĐỘ LỆCH - FIX API POLARS 0.52
    let (diff_percent, deviation_reason) = if !calculated_col.is_empty() && !measured_col.is_empty()
    {
        let total_calc = sum_column(&df, &calculated_col);
        let total_meas = sum_column(&df, &measured_col);

        if total_calc > 0.0 {
            let diff = ((total_meas - total_calc) / total_calc).abs() * 100.0;
            (diff, format!("Lệch {:.1}%", diff))
        } else {
            (0.0, "Không có dữ liệu tính toán".to_string())
        }
    } else {
        (0.0, "Thiếu cột tính toán/đo lường".to_string())
    };

    // 🎯 4. PHÁT HIỆN RỦI RO
    let (top_risks, high_risk_count) = detect_risks(&df, &column_names);

    // 🎯 5. TÍNH TOÁN TIẾN ĐỘ THANH TOÁN
    let payment_progress = calculate_payment_progress(&df, &column_names);

    // 🎯 6. XÁC ĐỊNH TRẠNG THÁI (với profit margin)
    let profit_margin_percent = payment_progress.profit_percent;
    let (status, status_reason) =
        determine_project_status(diff_percent, profit_margin_percent, high_risk_count, total_rows);

    // 🎯 7. HÀNH ĐỘNG ĐỀ XUẤT
    let pending_actions =
        suggest_actions(diff_percent, high_risk_count, &column_names, &status_col);

    // 🎯 8. TÍNH critical_count (rủi ro trên 15%)
    let critical_count = if diff_percent >= 15.0 {
        high_risk_count
    } else {
        0
    };

    Ok(DashboardSummary {
        status,
        status_reason: format!("{} - {}", status_reason, deviation_reason),
        top_risks,
        payment_progress,
        pending_actions,
        metrics: DashboardMetrics {
            total_rows,
            total_amount,
            avg_deviation: diff_percent,
            high_risk_count,
            critical_count,
            profit_margin_percent,
            last_updated: chrono::Local::now().to_rfc3339(),
        },
    })
}

/// Helper: Sum a column with Polars 0.52 API
fn sum_column(df: &DataFrame, col_name: &str) -> f64 {
    if let Ok(col) = df.column(col_name) {
        // ĐÚNG CÁCH Polars 0.52: Column -> owned Series -> cast -> sum
        col.as_materialized_series()
            .cast(&DataType::Float64)
            .unwrap_or(col.as_materialized_series().clone())
            .sum::<f64>()
            .unwrap_or(0.0)
    } else {
        0.0
    }
}

/// Tự động phát hiện cột
fn detect_columns(columns: &[String]) -> (String, String, String, String) {
    let mut amount_col = String::new();
    let mut calculated_col = String::new();
    let mut measured_col = String::new();
    let mut status_col = String::new();

    for col in columns {
        let col_lower = col.to_lowercase();

        if col_lower.contains("thành tiền")
            || col_lower.contains("thanh_tien")
            || col_lower.contains("tong_cong")
            || col_lower.contains("tổng cộng")
        {
            amount_col = col.clone();
        }
        if col_lower.contains("tính toán")
            || col_lower.contains("tinh_toan")
            || col_lower.contains("dự toán")
            || col_lower.contains("du_toan")
        {
            calculated_col = col.clone();
        }
        if col_lower.contains("đo lường")
            || col_lower.contains("do_luong")
            || col_lower.contains("thực tế")
            || col_lower.contains("thuc_te")
        {
            measured_col = col.clone();
        }
        if col_lower.contains("trạng thái")
            || col_lower.contains("trang_thai")
            || col_lower.contains("status")
        {
            status_col = col.clone();
        }
    }

    (amount_col, calculated_col, measured_col, status_col)
}

/// Phát hiện rủi ro - FIX API POLARS 0.52
fn detect_risks(df: &DataFrame, columns: &[String]) -> (Vec<RiskItem>, usize) {
    let mut risks = Vec::new();
    let mut high_risk_count = 0;

    for col in columns {
        let col_lower = col.to_lowercase();
        if col_lower.contains("lệch")
            || col_lower.contains("lech")
            || col_lower.contains("chênh")
            || col_lower.contains("chenh")
        {
            if let Ok(column) = df.column(col) {
                // ĐÚNG CÁCH với Polars 0.52
                let series = column.as_materialized_series();
                if let Ok(float_series) = series.f64() {
                    for i in 0..float_series.len().min(1000) {
                        if let Some(val) = float_series.get(i) {
                            if val.abs() > 10.0 {
                                high_risk_count += 1;

                                if risks.len() < 5 {
                                    risks.push(RiskItem {
                                        description: format!("Hạng mục {}", i + 1),
                                        deviation: val,
                                        impact: format!("{:.0} VND", val * 1_000_000.0),
                                        severity: if val.abs() > 20.0 {
                                            "HIGH".to_string()
                                        } else {
                                            "MEDIUM".to_string()
                                        },
                                    });
                                }
                            }
                        }
                    }
                }
            }
            break;
        }
    }

    if risks.is_empty() {
        risks.push(RiskItem {
            description: "Không phát hiện lệch lớn".to_string(),
            deviation: 0.0,
            impact: "0 VND".to_string(),
            severity: "LOW".to_string(),
        });
    }

    (risks, high_risk_count)
}

/// Xác định trạng thái dự án
fn determine_project_status(
    diff_percent: f64,
    profit_margin_percent: f64,
    high_risk_count: usize,
    total_rows: usize,
) -> (String, String) {
    let _risk_density = if total_rows > 0 {
        high_risk_count as f64 / total_rows as f64 * 100.0
    } else {
        0.0
    };

    // LOGIC KHỚP SPEC V2.5 (với status tiếng Việt cho thị trường VN)
    
    // ĐỎ (CRITICAL): Lệch >= 15% HOẶC nhiều rủi ro HOẶC lỗ
    if diff_percent >= 15.0 || high_risk_count >= 5 || profit_margin_percent <= 0.0 {
        // Special case for Quotation (only revenue data)
        if diff_percent == 0.0 && high_risk_count == 0 && profit_margin_percent == 0.0 {
             return (
                "BÁO GIÁ".to_string(),
                "CHỈ CÓ DỮ LIỆU BÁO GIÁ - CHƯA CÓ SỐ LIỆU THI CÔNG".to_string(),
            );
        }

        return (
            "ĐỎ".to_string(),
            format!(
                "Nguy cơ - Lệch {:.1}%, nhiều rủi ro ({}), lãi {:.1}%",
                diff_percent, high_risk_count, profit_margin_percent
            ),
        );
    }
    
    // XANH (SAFE): Lệch < 5%, không rủi ro, lãi > 10%
    if diff_percent < 5.0 && high_risk_count == 0 && profit_margin_percent > 10.0 {
        return (
            "XANH".to_string(),
            format!(
                "Ổn định - Lệch {:.1}%, lãi {:.1}%",
                diff_percent, profit_margin_percent
            ),
        );
    }

    // VÀNG (WARNING): Lệch 5-15% HOẶC rủi ro vừa HOẶC lãi thấp
    // Mọi trường hợp khác không phải ĐỎ hoặc XANH sẽ là VÀNG
    (
        "VÀNG".to_string(),
        format!(
            "Cần theo dõi - Lệch {:.1}%, rủi ro {}, lãi {:.1}%",
            diff_percent, high_risk_count, profit_margin_percent
        ),
    )
}

/// Đề xuất hành động
fn suggest_actions(
    diff_percent: f64,
    high_risk_count: usize,
    _columns: &[String],
    status_col: &str,
) -> Vec<String> {
    let mut actions = Vec::new();

    if diff_percent > 5.0 {
        actions.push("Kiểm tra lại khối lượng bê tông/thép".to_string());
        actions.push("Đối chiếu đơn giá với hợp đồng".to_string());
    }

    if high_risk_count > 0 {
        actions.push("Xem xét lại các hạng mục lệch >10%".to_string());
    }

    if !status_col.is_empty() {
        actions.push("Xử lý các hạng mục chưa duyệt".to_string());
    }

    if actions.is_empty() {
        actions.push("Tiếp tục thi công theo tiến độ".to_string());
        actions.push("Cập nhật biên bản nghiệm thu".to_string());
    }

    actions.truncate(3);
    actions
}

/// Tính toán tiến độ thanh toán - FIX API POLARS 0.52
fn calculate_payment_progress(df: &DataFrame, columns: &[String]) -> PaymentProgress {
    let mut received = 0.0;
    let mut total_contract = 0.0;

    for col in columns {
        let col_lower = col.to_lowercase();

        if col_lower.contains("đã thanh toán") || col_lower.contains("da_thanh_toan") {
            received = sum_column(df, col);
        }

        if col_lower.contains("tổng hợp đồng")
            || col_lower.contains("tong_hop_dong")
            || col_lower.contains("giá trị hợp đồng")
        {
            total_contract = sum_column(df, col);
        }
    }

    // Honest Mode: No more hardcoded fallbacks.
    // If columns are missing, we respect the reality (0.0).

    let percent = if total_contract > 0.0 {
        received / total_contract * 100.0
    } else {
        0.0
    };

    let projected_profit = total_contract * 0.15 - (total_contract - received) * 0.1;
    let profit_percent = if total_contract > 0.0 {
        projected_profit / total_contract * 100.0
    } else {
        0.0
    };

    PaymentProgress {
        received,
        total_contract,
        percent,
        projected_profit,
        profit_percent,
    }
}

// ============================================================================
// 🧪 TEST SUITE V2.5.1 - BUSINESS RULES VALIDATION
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- TEST 1: Project Status Logic ---

    #[test]
    fn test_determine_project_status_green_perfect() {
        // Lệch <5%, không rủi ro, lãi >10% -> XANH
        let (status, reason) = determine_project_status(0.5, 15.0, 0, 100);
        assert_eq!(status, "XANH");
        assert!(reason.contains("Ổn định"));
    }

    #[test]
    fn test_determine_project_status_green_low_risk() {
        // Lệch 4%, ít rủi ro, lãi tốt -> XANH
        let (status, _reason) = determine_project_status(4.0, 12.0, 0, 100);
        assert_eq!(status, "XANH");
    }

    #[test]
    fn test_determine_project_status_yellow_medium() {
        // Lệch 8% (5-15%) -> VÀNG
        let (status, reason) = determine_project_status(8.0, 12.0, 2, 100);
        assert_eq!(status, "VÀNG");
        assert!(reason.contains("Cần theo dõi"));
    }

    #[test]
    fn test_determine_project_status_red_by_deviation() {
        // Lệch 15% -> ĐỎ (threshold changed from 20% to 15%)
        let (status, _reason) = determine_project_status(15.0, 12.0, 3, 100);
        assert_eq!(status, "ĐỎ");
    }

    #[test]
    fn test_determine_project_status_red_by_loss() {
        // Lỗ (profit <= 0) -> ĐỎ
        let (status, reason) = determine_project_status(8.0, -2.0, 2, 100);
        assert_eq!(status, "ĐỎ");
        assert!(reason.contains("Nguy cơ"));
    }

    #[test]
    fn test_determine_project_status_edge_case_zero_rows() {
        // Edge case: lệch nhẹ, có lãi
        let (status, _) = determine_project_status(5.0, 12.0, 0, 0);
        // Với 5% và profit 12%, nên là VÀNG (>= 5%)
        assert_eq!(status, "VÀNG");
    }

    // --- TEST 2: Column Detection (Vietnamese) ---

    #[test]
    fn test_detect_columns_vietnamese_standard() {
        let columns = vec![
            "STT".to_string(),
            "Thành tiền".to_string(),
            "Tính toán".to_string(),
            "Đo lường".to_string(),
            "Trạng thái".to_string(),
        ];
        let (amount, calc, meas, status) = detect_columns(&columns);
        assert_eq!(amount, "Thành tiền");
        assert_eq!(calc, "Tính toán");
        assert_eq!(meas, "Đo lường");
        assert_eq!(status, "Trạng thái");
    }

    #[test]
    fn test_detect_columns_vietnamese_variants() {
        let columns = vec![
            "tổng cộng".to_string(),
            "dự toán".to_string(),
            "thực tế".to_string(),
            "status".to_string(),
        ];
        let (amount, calc, meas, status) = detect_columns(&columns);
        assert_eq!(amount, "tổng cộng");
        assert_eq!(calc, "dự toán");
        assert_eq!(meas, "thực tế");
        assert_eq!(status, "status");
    }

    #[test]
    fn test_detect_columns_ascii_variants() {
        let columns = vec![
            "thanh_tien".to_string(),
            "tinh_toan".to_string(),
            "do_luong".to_string(),
            "trang_thai".to_string(),
        ];
        let (amount, calc, meas, status) = detect_columns(&columns);
        assert_eq!(amount, "thanh_tien");
        assert_eq!(calc, "tinh_toan");
        assert_eq!(meas, "do_luong");
        assert_eq!(status, "trang_thai");
    }

    #[test]
    fn test_detect_columns_missing() {
        // Trường hợp không tìm thấy cột nào
        let columns = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let (amount, calc, meas, status) = detect_columns(&columns);
        assert!(amount.is_empty());
        assert!(calc.is_empty());
        assert!(meas.is_empty());
        assert!(status.is_empty());
    }

    // --- TEST 3: Sum Column (requires DataFrame) ---

    #[test]
    fn test_sum_column_numeric() {
        // Tạo DataFrame test
        let values = Series::new("test_col".into(), vec![100.0, 200.0, 300.0]);
        let df = DataFrame::new(vec![values.into()]).unwrap();

        let result = sum_column(&df, "test_col");
        assert_eq!(result, 600.0);
    }

    #[test]
    fn test_sum_column_missing() {
        // Cột không tồn tại
        let values = Series::new("test_col".into(), vec![100.0, 200.0]);
        let df = DataFrame::new(vec![values.into()]).unwrap();

        let result = sum_column(&df, "nonexistent");
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_sum_column_empty() {
        // DataFrame rỗng
        let values: Vec<f64> = vec![];
        let series = Series::new("empty".into(), values);
        let df = DataFrame::new(vec![series.into()]).unwrap();

        let result = sum_column(&df, "empty");
        assert_eq!(result, 0.0);
    }

    // --- TEST 4: Detect Risks ---

    #[test]
    fn test_detect_risks_with_deviations() {
        // Tạo DataFrame với cột "Lệch %"
        let deviations = Series::new("Lệch %".into(), vec![15.0, 25.0, 5.0, 30.0]);
        let df = DataFrame::new(vec![deviations.into()]).unwrap();
        let columns = df
            .get_column_names()
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        let (risks, high_risk_count) = detect_risks(&df, &columns);

        // Có 3 dòng lệch >10%
        assert_eq!(high_risk_count, 3);
        // Trả về tối đa 3 risk items
        assert!(risks.len() >= 1);
        assert!(risks.len() <= 3);
    }

    #[test]
    fn test_detect_risks_no_deviation_column() {
        // Không có cột "Lệch"
        let values = Series::new("normal_col".into(), vec![1.0, 2.0, 3.0]);
        let df = DataFrame::new(vec![values.into()]).unwrap();
        let columns = df
            .get_column_names()
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        let (risks, high_risk_count) = detect_risks(&df, &columns);

        assert_eq!(high_risk_count, 0);
        assert_eq!(risks.len(), 1);
        assert_eq!(risks[0].description, "Không phát hiện lệch lớn");
        assert_eq!(risks[0].severity, "LOW");
    }

    // --- TEST 5: Suggest Actions ---

    #[test]
    fn test_suggest_actions_green() {
        // Green status: diff < 5%, no risks, no status column
        let actions = suggest_actions(2.0, 0, &[], "");

        println!("DEBUG Green actions: {:?}", actions);

        assert!(!actions.is_empty());
        // With diff < 5%, no risks, empty status_col -> returns default actions
        assert!(actions[0].contains("Tiếp tục") || actions[0].contains("Cập nhật"));
    }

    #[test]
    fn test_suggest_actions_yellow() {
        let actions = suggest_actions(12.0, 10, &[], "status");
        assert!(actions.len() >= 2);
        assert!(
            actions
                .iter()
                .any(|a| a.contains("Kiểm tra") || a.contains("Xem xét"))
        );
    }

    #[test]
    fn test_suggest_actions_red() {
        let actions = suggest_actions(25.0, 30, &[], "status");
        assert!(actions.len() >= 2);
        assert!(
            actions
                .iter()
                .any(|a| a.contains("Kiểm tra") || a.contains("Xem xét"))
        );
    }
}
