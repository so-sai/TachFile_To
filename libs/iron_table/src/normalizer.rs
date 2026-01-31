use crate::contract::CellValue;

/// Normalizes a header string according to Iron Truth Contract 4.1.
/// - Lowercase
/// - Remove accents
/// - Trim whitespace
/// - Replace spaces with underscores
pub fn normalize_header(raw: &str) -> String {
    let lower = raw.trim().to_lowercase();
    let no_accent: String = lower.chars().map(remove_vietnamese_accent).collect();
    no_accent.split_whitespace().collect::<Vec<&str>>().join("_")
}

/// Normalizes a number to 3 decimal places (Contract 4.3).
pub fn normalize_number(raw: f64) -> f64 {
    (raw * 1000.0).round() / 1000.0
}

/// Normalizes units to standard formats (Contract 4.2).
/// Only exact matches are allowed.
pub fn normalize_unit(raw: &str) -> Option<String> {
    match raw.trim().to_lowercase().as_str() {
        "m2" | "mét vuông" => Some("m²".to_string()),
        "m3" | "mét khối" => Some("m³".to_string()),
        "cái" => Some("cái".to_string()),
        "bộ" => Some("bộ".to_string()),
        "kg" => Some("kg".to_string()),
        "tan" | "tấn" => Some("tấn".to_string()),
        _ => None,
    }
}

/// Normalizes null/empty values (Contract 4.4).
pub fn normalize_null(raw: &str) -> CellValue {
    match raw.trim() {
        "" | "-" | " " => CellValue::Null,
        "0" => CellValue::Float(0.0),
        val => CellValue::Text(val.to_string()),
    }
}

fn remove_vietnamese_accent(c: char) -> char {
    match c {
        'à'|'á'|'ạ'|'ả'|'ã'|'â'|'ầ'|'ấ'|'ậ'|'ẩ'|'ẫ'|'ă'|'ằ'|'ắ'|'ặ'|'ẳ'|'ẵ' => 'a',
        'è'|'é'|'ẹ'|'ẻ'|'ẽ'|'ê'|'ề'|'ế'|'ệ'|'ể'|'ễ' => 'e',
        'ì'|'í'|'ị'|'ỉ'|'ĩ' => 'i',
        'ò'|'ó'|'ọ'|'ỏ'|'õ'|'ô'|'ồ'|'ố'|'ộ'|'ổ'|'ỗ'|'ơ'|'ờ'|'ớ'|'ợ'|'ở'|'ỡ' => 'o',
        'ù'|'ú'|'ụ'|'ủ'|'ũ'|'ư'|'ừ'|'ứ'|'ự'|'ử'|'ữ' => 'u',
        'ỳ'|'ý'|'ỵ'|'ỷ'|'ỹ' => 'y',
        'đ' => 'd',
        _ => c,
    }
}
