use regex::Regex;

/// The NumericSanitizer is responsible for cleaning up messy OCR numbers
/// commonly found in Vietnamese construction documents, converting them reliably into `f64`.
pub struct NumericSanitizer;

impl NumericSanitizer {
    /// Attempts to parse a raw string into an `f64`.
    /// Handles Vietnamese number formats (e.g. `1.250.000,50` or `1 250 000.50`),
    /// fixes common OCR mistakes (like 'l' to '1', 'o' to '0'), and strips wrappers like `()`.
    pub fn sanitize(raw: &str) -> Option<f64> {
        let mut text = raw.trim().to_string();
        if text.is_empty() {
            return None;
        }

        // 1. Strip common wrappers like parentheses for negative numbers or brackets
        let is_negative = text.starts_with('(') && text.ends_with(')') || text.starts_with('-');
        text = text
            .trim_matches(|c| c == '(' || c == ')' || c == '[' || c == ']' || c == '-')
            .to_string();

        // 2. Fix common OCR mistakes
        text = text
            .replace(['l', 'I'], "1")
            .replace(['O', 'o'], "0")
            .replace('S', "5");

        // 3. Keep only digits, periods, commas, and spaces
        let re = Regex::new(r"[^\d.,\s]").unwrap();
        text = re.replace_all(&text, "").to_string();

        // 4. Handle thousands separators vs decimal points
        // In VN, 1.000.000,50 is common. In US, 1,000,000.50 is common.
        // We find the last separator to guess the format.
        let last_comma = text.rfind(',');
        let last_period = text.rfind('.');

        let standardized = match (last_comma, last_period) {
            (Some(c), Some(p)) if c > p => {
                // Comma is the decimal separator: "1.000.000,50" -> "1000000.50"
                text.replace('.', "").replace(',', ".")
            }
            (Some(c), Some(p)) if p > c => {
                // Period is the decimal separator: "1,000,000.50" -> "1000000.50"
                text.replace(',', "")
            }
            (Some(_), None) => {
                // Only commas. If there's only one and it has 1-2 digits after, it's likely a decimal.
                let parts: Vec<&str> = text.split(',').collect();
                if parts.len() == 2 && parts[1].len() <= 2 {
                    text.replace(',', ".") // "1000,50" -> "1000.50"
                } else {
                    text.replace(',', "") // "1,000,000" -> "1000000"
                }
            }
            (None, Some(_)) => {
                // Only periods. If there's only one and it has 1-2 digits after, it's likely a decimal.
                let parts: Vec<&str> = text.split('.').collect();
                if parts.len() == 2 && parts[1].len() <= 2 {
                    text // Keep as is: "1000.50"
                } else {
                    text.replace('.', "") // "1.000.000" -> "1000000"
                }
            }
            _ => text.replace(' ', ""), // Just digits, strip any spaces
        };

        // Final cleanup of remaining spaces
        let standardized = standardized.replace(' ', "");

        if standardized.is_empty() {
            return None;
        }

        // 5. Parse to f64
        if let Ok(mut val) = standardized.parse::<f64>() {
            if is_negative {
                val = -val;
            }
            Some(val)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_sanitizer_vn_format() {
        // VN Standard: Dot for thousands, Comma for decimal
        assert_eq!(NumericSanitizer::sanitize("1.250.000,50"), Some(1250000.5));
        assert_eq!(NumericSanitizer::sanitize("1.000,5"), Some(1000.5));
        assert_eq!(NumericSanitizer::sanitize("500,00"), Some(500.0));
    }

    #[test]
    fn test_numeric_sanitizer_us_format() {
        // US Standard: Comma for thousands, Dot for decimal
        assert_eq!(NumericSanitizer::sanitize("1,250,000.50"), Some(1250000.5));
        assert_eq!(NumericSanitizer::sanitize("1,000.5"), Some(1000.5));
        assert_eq!(NumericSanitizer::sanitize("500.00"), Some(500.0));
    }

    #[test]
    fn test_numeric_sanitizer_spaces() {
        // Spaces as thousands separators
        assert_eq!(NumericSanitizer::sanitize("1 500 000"), Some(1500000.0));
        assert_eq!(NumericSanitizer::sanitize("1 250 000.50"), Some(1250000.5));
        assert_eq!(NumericSanitizer::sanitize("1 250 000,50"), Some(1250000.5));
    }

    #[test]
    fn test_numeric_sanitizer_ocr_noise() {
        // Common OCR mistakes (o/O -> 0, l/I -> 1, S -> 5)
        assert_eq!(NumericSanitizer::sanitize("l.2S0.000,5o"), Some(1250000.5)); // l -> 1, S -> 5, o -> 0
        assert_eq!(NumericSanitizer::sanitize("o.50"), Some(0.5)); // o -> 0
        assert_eq!(NumericSanitizer::sanitize("I.000"), Some(1000.0)); // I -> 1
    }

    #[test]
    fn test_numeric_sanitizer_negative_wrappers() {
        // Accounting negative formats
        assert_eq!(
            NumericSanitizer::sanitize("(1.250.000,50)"),
            Some(-1250000.5)
        );
        assert_eq!(NumericSanitizer::sanitize("-1,500.00"), Some(-1500.0));
        assert_eq!(NumericSanitizer::sanitize("[500]"), Some(500.0));
    }

    #[test]
    fn test_numeric_sanitizer_invalid() {
        // Pure text should return None
        assert_eq!(NumericSanitizer::sanitize("Không có số"), None);
        assert_eq!(NumericSanitizer::sanitize(""), None);
        assert_eq!(NumericSanitizer::sanitize("   "), None);
    }
}
