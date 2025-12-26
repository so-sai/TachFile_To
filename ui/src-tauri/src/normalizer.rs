/*
 * TACHFILETO ENTERPRISE - TERMINOLOGY NORMALIZER V1.0
 * ==================================================
 * Chức năng: Chuẩn hóa thuật ngữ kế toán/xây dựng Tiếng Việt
 * Thuật toán: Jaro-Winkler Fuzzy Matching (ngưỡng 0.85)
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strsim::jaro_winkler;
use tauri::command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedTerm {
    pub original: String,
    pub standardized: String,
    pub category: String,
}

#[command]
pub fn cmd_normalize_descriptions(
    descriptions: Vec<String>,
) -> Result<Vec<NormalizedTerm>, String> {
    let result = descriptions
        .iter()
        .map(|desc| GLOBAL_NORMALIZER.normalize(desc))
        .collect();
    Ok(result)
}

pub struct TerminologyNormalizer {
    // Map biến thể -> Chuẩn
    term_mapping: HashMap<String, String>,
    // Map Chuẩn -> Nhóm (material, labor, etc.)
    category_mapping: HashMap<String, String>,
}

impl TerminologyNormalizer {
    pub fn new() -> Self {
        let mut term_mapping = HashMap::new();

        // --- CHI PHÍ NGUYÊN VẬT LIỆU ---
        term_mapping.insert("cp nvl".to_string(), "chi phí nguyên vật liệu".to_string());
        term_mapping.insert(
            "cp nguyên vật liệu".to_string(),
            "chi phí nguyên vật liệu".to_string(),
        );
        term_mapping.insert(
            "nguyen vat lieu".to_string(),
            "chi phí nguyên vật liệu".to_string(),
        );
        term_mapping.insert("vật tư".to_string(), "chi phí nguyên vật liệu".to_string());

        // --- CHI PHÍ NHÂN CÔNG ---
        term_mapping.insert("cp nhân công".to_string(), "chi phí nhân công".to_string());
        term_mapping.insert("cp nhan cong".to_string(), "chi phí nhân công".to_string());
        term_mapping.insert("tiền công".to_string(), "chi phí nhân công".to_string());
        term_mapping.insert("tiền lương".to_string(), "chi phí nhân công".to_string());
        term_mapping.insert(
            "nhân công xây dựng".to_string(),
            "chi phí nhân công".to_string(),
        );

        // --- VẬN CHUYỂN ---
        term_mapping.insert("vc".to_string(), "vận chuyển".to_string());
        term_mapping.insert("cước vận chuyển".to_string(), "vận chuyển".to_string());
        term_mapping.insert("phí vận chuyển".to_string(), "vận chuyển".to_string());

        let mut category_mapping = HashMap::new();
        category_mapping.insert("chi phí nguyên vật liệu".to_string(), "vật tư".to_string());
        category_mapping.insert("chi phí nhân công".to_string(), "nhân công".to_string());
        category_mapping.insert("vận chuyển".to_string(), "logistics".to_string());

        Self {
            term_mapping,
            category_mapping,
        }
    }

    pub fn normalize(&self, description: &str) -> NormalizedTerm {
        let cleaned = self.clean_text(description);
        let standardized = self.standardize_term(&cleaned);
        let category = self.categorize(&standardized);

        NormalizedTerm {
            original: description.to_string(),
            standardized,
            category,
        }
    }

    fn clean_text(&self, text: &str) -> String {
        text.to_lowercase().trim().to_string()
    }

    fn standardize_term(&self, term: &str) -> String {
        if let Some(mapped) = self.term_mapping.get(term) {
            return mapped.clone();
        }

        let mut best_score = 0.82; // THRESHOLD: Lowered to 0.82 for better Vietnamese matching
        let mut best_match = None;

        for (key, value) in &self.term_mapping {
            let score = jaro_winkler(term, key);
            if score > best_score {
                best_score = score;
                best_match = Some(value.clone());
            }
        }

        best_match.unwrap_or_else(|| term.to_string())
    }

    fn categorize(&self, standardized_term: &str) -> String {
        self.category_mapping
            .get(standardized_term)
            .cloned()
            .unwrap_or_else(|| "khác".to_string())
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_NORMALIZER: TerminologyNormalizer = TerminologyNormalizer::new();
}

// ============================================================================
// 🧪 TEST SUITE V2.5.1 - TERMINOLOGY NORMALIZER
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_exact_match_nvl() {
        let normalizer = TerminologyNormalizer::new();
        let result = normalizer.normalize("cp nvl");

        assert_eq!(result.standardized, "chi phí nguyên vật liệu");
        assert_eq!(result.category, "vật tư");
        assert_eq!(result.original, "cp nvl");
    }

    #[test]
    fn test_normalize_exact_match_labor() {
        let normalizer = TerminologyNormalizer::new();
        let result = normalizer.normalize("tiền công");

        assert_eq!(result.standardized, "chi phí nhân công");
        assert_eq!(result.category, "nhân công");
    }

    #[test]
    fn test_normalize_fuzzy_match() {
        let normalizer = TerminologyNormalizer::new();

        // "cp nguyên vật liệu" gần với "cp nvl"
        let result = normalizer.normalize("cp nguyen vat lieu");
        assert_eq!(result.standardized, "chi phí nguyên vật liệu");
        assert_eq!(result.category, "vật tư");
    }

    #[test]
    fn test_normalize_case_insensitive() {
        let normalizer = TerminologyNormalizer::new();

        let result1 = normalizer.normalize("CP NVL");
        let result2 = normalizer.normalize("Cp Nvl");
        let result3 = normalizer.normalize("cp nvl");

        assert_eq!(result1.standardized, result2.standardized);
        assert_eq!(result2.standardized, result3.standardized);
    }

    #[test]
    fn test_normalize_whitespace_trimming() {
        let normalizer = TerminologyNormalizer::new();

        let result = normalizer.normalize("  tiền công  ");
        assert_eq!(result.standardized, "chi phí nhân công");
    }

    #[test]
    fn test_normalize_unknown_term() {
        let normalizer = TerminologyNormalizer::new();

        // Thuật ngữ hoàn toàn không match
        let result = normalizer.normalize("xyz abc 123");
        assert_eq!(result.standardized, "xyz abc 123");
        assert_eq!(result.category, "khác");
    }

    #[test]
    fn test_normalize_transport() {
        let normalizer = TerminologyNormalizer::new();

        let result1 = normalizer.normalize("vc");
        let result2 = normalizer.normalize("phí vận chuyển");

        assert_eq!(result1.standardized, "vận chuyển");
        assert_eq!(result1.category, "logistics");
        assert_eq!(result2.standardized, "vận chuyển");
        assert_eq!(result2.category, "logistics");
    }

    #[test]
    fn test_cmd_normalize_descriptions_batch() {
        let descriptions = vec![
            "cp nvl".to_string(),
            "tiền công".to_string(),
            "vc".to_string(),
        ];

        let result = cmd_normalize_descriptions(descriptions).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].standardized, "chi phí nguyên vật liệu");
        assert_eq!(result[1].standardized, "chi phí nhân công");
        assert_eq!(result[2].standardized, "vận chuyển");
    }

    #[test]
    fn test_jaro_winkler_threshold() {
        let normalizer = TerminologyNormalizer::new();

        // "cp nhan cong" should fuzzy match "cp nhân công" with threshold 0.82
        let result = normalizer.normalize("cp nhan cong");

        println!(
            "DEBUG: standardized='{}', category='{}'",
            result.standardized, result.category
        );

        // Should fuzzy match to labor cost
        assert_eq!(result.standardized, "chi phí nhân công");
        assert_eq!(result.category, "nhân công");
    }
}

// ============================================================================
// 📊 COLUMN HEADER NORMALIZER - DASHBOARD INTEGRATION
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnNormalizationResult {
    pub original_name: String,
    pub normalized_name: String,
    pub column_type: ColumnType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ColumnType {
    Amount,     // Cột tiền: thành tiền, tổng cộng, thanh_tien
    Calculated, // Cột tính toán: khối lượng tính toán, KLTT
    Measured,   // Cột đo lường: khối lượng thực tế, KLTT
    Status,     // Cột trạng thái: trang_thai, tinh_trang
    Other,      // Cột khác
}

impl TerminologyNormalizer {
    /// Chuẩn hóa tên cột cho Dashboard
    pub fn normalize_column_name(&self, column_name: &str) -> ColumnNormalizationResult {
        let cleaned = column_name.trim().to_lowercase();

        // Xóa ký tự đặc biệt nhưng giữ dấu tiếng Việt
        let cleaned = cleaned
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '_')
            .collect::<String>();

        // MAP cho Dashboard Detection
        let (normalized, col_type) = if cleaned.contains("thanh tien")
            || cleaned.contains("thành tiền")
            || cleaned.contains("tong cong")
            || cleaned.contains("tổng cộng")
            || cleaned.contains("tong_tien")
            || cleaned.contains("thanh_tien")
        {
            ("thanh_tien".to_string(), ColumnType::Amount)
        } else if cleaned.contains("tinh toan")
            || cleaned.contains("tính toán")
            || cleaned.contains("khoi luong tinh toan")
            || cleaned.contains("khối lượng tính toán")
            || cleaned.contains("du toan")
            || cleaned.contains("dự toán")
            || cleaned.contains("tinh_toan")
            || cleaned.contains("kltt")
        {
            ("khoi_luong_tinh_toan".to_string(), ColumnType::Calculated)
        } else if cleaned.contains("do luong")
            || cleaned.contains("đo lường")
            || cleaned.contains("khoi luong thuc te")
            || cleaned.contains("khối lượng thực tế")
            || cleaned.contains("thuc te")
            || cleaned.contains("thực tế")
            || cleaned.contains("do_luong")
            || cleaned.contains("kltt")
        {
            ("khoi_luong_thuc_te".to_string(), ColumnType::Measured)
        } else if cleaned.contains("trang thai")
            || cleaned.contains("trạng thái")
            || cleaned.contains("tinh trang")
            || cleaned.contains("tình trạng")
            || cleaned.contains("trang_thai")
            || cleaned.contains("status")
        {
            ("trang_thai".to_string(), ColumnType::Status)
        } else {
            // Giữ nguyên nhưng chuẩn hóa format
            let normalized = cleaned.replace(" ", "_").replace("-", "_");
            (normalized, ColumnType::Other)
        };

        ColumnNormalizationResult {
            original_name: column_name.to_string(),
            normalized_name: normalized,
            column_type: col_type,
        }
    }

    /// Chuẩn hóa toàn bộ DataFrame columns
    pub fn normalize_dataframe_columns(
        &self,
        df: &mut polars::prelude::DataFrame,
    ) -> Result<Vec<ColumnNormalizationResult>, String> {
        use polars::prelude::*;

        let original_cols: Vec<String> = df
            .get_column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();

        let mut results = Vec::new();
        let mut rename_map = Vec::new();

        for col in &original_cols {
            let norm_result = self.normalize_column_name(col);
            results.push(norm_result.clone());

            if norm_result.normalized_name != *col {
                rename_map.push((col.clone(), norm_result.normalized_name.clone()));
            }
        }

        // Áp dụng rename cho DataFrame
        for (old_name, new_name) in rename_map {
            if df.column(&old_name).is_ok() {
                let _ = df.rename(&old_name, new_name.as_str().into());
                println!("[Column Normalizer] '{}' → '{}'", old_name, new_name);
            }
        }

        Ok(results)
    }
}

/// Tauri command - Normalize column names
#[command]
pub fn cmd_normalize_columns(
    column_names: Vec<String>,
) -> Result<Vec<ColumnNormalizationResult>, String> {
    let result = column_names
        .iter()
        .map(|col| GLOBAL_NORMALIZER.normalize_column_name(col))
        .collect();
    Ok(result)
}

// ============================================================================
// 🧪 TEST SUITE - COLUMN NORMALIZER
// ============================================================================

#[cfg(test)]
mod column_tests {
    use super::*;

    #[test]
    fn test_normalize_column_amount() {
        let normalizer = TerminologyNormalizer::new();

        let result = normalizer.normalize_column_name("Thành tiền (VNĐ)");
        assert_eq!(result.normalized_name, "thanh_tien");
        assert_eq!(result.column_type, ColumnType::Amount);
    }

    #[test]
    fn test_normalize_column_calculated() {
        let normalizer = TerminologyNormalizer::new();

        let result = normalizer.normalize_column_name("KL Tính Toán");
        assert_eq!(result.normalized_name, "khoi_luong_tinh_toan");
        assert_eq!(result.column_type, ColumnType::Calculated);
    }

    #[test]
    fn test_normalize_column_measured() {
        let normalizer = TerminologyNormalizer::new();

        let result = normalizer.normalize_column_name("Đo Lường Thực Tế");
        assert_eq!(result.normalized_name, "khoi_luong_thuc_te");
        assert_eq!(result.column_type, ColumnType::Measured);
    }

    #[test]
    fn test_normalize_column_status() {
        let normalizer = TerminologyNormalizer::new();

        let result = normalizer.normalize_column_name("Trạng Thái");
        assert_eq!(result.normalized_name, "trang_thai");
        assert_eq!(result.column_type, ColumnType::Status);
    }

    #[test]
    fn test_normalize_column_other() {
        let normalizer = TerminologyNormalizer::new();

        let result = normalizer.normalize_column_name("Ghi Chú");
        assert_eq!(result.column_type, ColumnType::Other);
        assert!(result.normalized_name.contains("ghi"));
    }

    #[test]
    fn test_cmd_normalize_columns() {
        let columns = vec![
            "Thành tiền".to_string(),
            "Tính toán".to_string(),
            "Status".to_string(),
        ];

        let results = cmd_normalize_columns(columns).unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].column_type, ColumnType::Amount);
        assert_eq!(results[1].column_type, ColumnType::Calculated);
        assert_eq!(results[2].column_type, ColumnType::Status);
    }
}
