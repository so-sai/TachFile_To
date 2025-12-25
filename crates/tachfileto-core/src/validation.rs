//! Validation Module - Vietnamese Construction Rules v1.2
//! Implements: VAT calculation, quantity validation with edge cases, fuzzy matching

use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// VAT calculation for Vietnamese construction invoices
#[derive(Debug, Clone)]
pub struct PriceCalculation {
    pub subtotal: Decimal,
    pub vat_rate: Decimal,
    pub vat_amount: Decimal,
    pub grand_total: Decimal,
}

impl PriceCalculation {
    /// Calculate VAT with Vietnamese rounding rules
    /// - VAT amount: round to 0 decimals (MidpointAwayFromZero)
    /// - Grand total: round to nearest 1,000 VND
    pub fn calculate(subtotal: Decimal, vat_rate: Decimal) -> Self {
        // Step 1: Calculate VAT (round to 0 decimals)
        let vat_amount = (subtotal * vat_rate).round_dp(0);

        // Step 2: Calculate total
        let raw_total = subtotal + vat_amount;

        // Step 3: Round to nearest 1,000 VND
        let grand_total = (raw_total / dec!(1000)).round() * dec!(1000);

        Self {
            subtotal,
            vat_rate,
            vat_amount,
            grand_total,
        }
    }

    /// Default 10% VAT rate for Vietnam
    pub fn with_default_vat(subtotal: Decimal) -> Self {
        Self::calculate(subtotal, dec!(0.10))
    }
}

/// Validation result for quantity checks
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    Pass,
    Fail {
        reason: String,
        diff_percent: Option<f64>,
    },
}

/// Advanced quantity validation with edge case handling
/// - Near-zero: uses absolute difference instead of percentage
/// - Small quantities: combines % tolerance with absolute minimum
pub fn validate_quantity(
    calculated: f64,
    measured: f64,
    tolerance_percent: f64,
    min_absolute_threshold: f64,
) -> ValidationResult {
    // Edge case: near-zero calculated value
    if calculated.abs() < min_absolute_threshold {
        let abs_diff = (measured - calculated).abs();
        if abs_diff <= min_absolute_threshold {
            return ValidationResult::Pass;
        } else {
            return ValidationResult::Fail {
                reason: format!(
                    "Absolute diff {:.4} exceeded threshold {:.4} (near-zero case)",
                    abs_diff, min_absolute_threshold
                ),
                diff_percent: None,
            };
        }
    }

    // Standard percentage check
    let diff_percent = ((measured - calculated) / calculated).abs() * 100.0;

    if diff_percent <= tolerance_percent {
        ValidationResult::Pass
    } else {
        ValidationResult::Fail {
            reason: format!(
                "Diff {:.2}% exceeded tolerance {:.2}%",
                diff_percent, tolerance_percent
            ),
            diff_percent: Some(diff_percent),
        }
    }
}

/// Jaro-Winkler similarity for fuzzy column matching
pub fn jaro_winkler(s1: &str, s2: &str) -> f64 {
    let s1_lower = s1.to_lowercase();
    let s2_lower = s2.to_lowercase();

    if s1_lower == s2_lower {
        return 1.0;
    }

    let len1 = s1_lower.chars().count();
    let len2 = s2_lower.chars().count();

    if len1 == 0 || len2 == 0 {
        return 0.0;
    }

    // Jaro distance
    let match_distance = (std::cmp::max(len1, len2) / 2).saturating_sub(1);

    let s1_chars: Vec<char> = s1_lower.chars().collect();
    let s2_chars: Vec<char> = s2_lower.chars().collect();

    let mut s1_matches = vec![false; len1];
    let mut s2_matches = vec![false; len2];

    let mut matches = 0;
    let mut transpositions = 0;

    // Find matches
    for i in 0..len1 {
        let start = i.saturating_sub(match_distance);
        let end = std::cmp::min(i + match_distance + 1, len2);

        for j in start..end {
            if s2_matches[j] || s1_chars[i] != s2_chars[j] {
                continue;
            }
            s1_matches[i] = true;
            s2_matches[j] = true;
            matches += 1;
            break;
        }
    }

    if matches == 0 {
        return 0.0;
    }

    // Count transpositions
    let mut k = 0;
    for i in 0..len1 {
        if !s1_matches[i] {
            continue;
        }
        while !s2_matches[k] {
            k += 1;
        }
        if s1_chars[i] != s2_chars[k] {
            transpositions += 1;
        }
        k += 1;
    }

    let jaro = (matches as f64 / len1 as f64
        + matches as f64 / len2 as f64
        + (matches - transpositions / 2) as f64 / matches as f64)
        / 3.0;

    // Winkler modification (prefix bonus)
    let mut prefix_len = 0;
    for (c1, c2) in s1_chars.iter().zip(s2_chars.iter()).take(4) {
        if c1 == c2 {
            prefix_len += 1;
        } else {
            break;
        }
    }

    jaro + (prefix_len as f64 * 0.1 * (1.0 - jaro))
}

/// Find best matching column by keywords using fuzzy matching
pub fn find_column_fuzzy(headers: &[String], keywords: &[&str], threshold: f64) -> Option<usize> {
    headers
        .iter()
        .enumerate()
        .filter_map(|(i, header)| {
            let max_sim = keywords
                .iter()
                .map(|kw| jaro_winkler(header, kw))
                .fold(0.0_f64, |a, b| a.max(b));

            if max_sim >= threshold {
                Some((i, max_sim))
            } else {
                None
            }
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vat_calculation() {
        let calc = PriceCalculation::calculate(Decimal::from_str("12345678").unwrap(), dec!(0.10));

        assert_eq!(calc.vat_amount, Decimal::from_str("1234568").unwrap());
        assert_eq!(calc.grand_total, Decimal::from_str("13580000").unwrap());
    }

    #[test]
    fn test_near_zero_validation() {
        // Near-zero case
        let result = validate_quantity(0.0, 0.05, 5.0, 0.1);
        assert_eq!(result, ValidationResult::Pass);

        // Near-zero fail
        let result = validate_quantity(0.0, 0.2, 5.0, 0.1);
        assert!(matches!(result, ValidationResult::Fail { .. }));
    }

    #[test]
    fn test_fuzzy_matching() {
        // Typo in header
        let sim = jaro_winkler("Khói lượng", "Khối lượng");
        assert!(sim > 0.8);

        // Find column
        let headers = vec!["STT".to_string(), "Khói lượng".to_string()];
        let idx = find_column_fuzzy(&headers, &["Khối lượng"], 0.8);
        assert_eq!(idx, Some(1));
    }
}
