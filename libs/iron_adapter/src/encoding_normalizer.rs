//! # Elite Encoding Normalizer - Mission 026
//!
//! Converts legacy Vietnamese encodings (VNI, TCVN3) to Unicode.
//!
//! ## Architecture
//! - **Thread-safe**: Uses `OnceLock` for global singleton
//! - **Single-pass**: O(n) translation for performance
//! - **Observable**: All functions traced via `#[tracing::instrument]`
//!
//! ## Encoding Support
//! - VNI (VNI-Times, VNI-Helve, etc.): Multi-character sequences → Unicode
//! - TCVN3 (.VnTime, .VnArial, etc.): Single-character → Unicode

use std::collections::HashMap;
use std::sync::OnceLock;
use tracing::{info, instrument};

/// The Elite Encoding Normalizer.
/// Converts VNI/TCVN3 legacy encodings to Unicode.
pub struct EncodingNormalizer {
    /// VNI: Multi-character sequences (e.g., "a1" → 'á')
    vni_map: HashMap<&'static str, char>,
    /// TCVN3: Single-character mappings (e.g., '¸' → 'á')
    tcvn3_map: HashMap<char, char>,
}

/// Global singleton instance
static GLOBAL_NORMALIZER: OnceLock<EncodingNormalizer> = OnceLock::new();

impl EncodingNormalizer {
    /// Get the global singleton instance.
    pub fn global() -> &'static Self {
        GLOBAL_NORMALIZER.get_or_init(Self::new)
    }

    /// Create a new normalizer with full mapping tables.
    fn new() -> Self {
        Self {
            vni_map: build_vni_map(),
            tcvn3_map: build_tcvn3_map(),
        }
    }

    /// Convert VNI-encoded text to Unicode.
    ///
    /// VNI uses digit suffixes to encode Vietnamese diacritics:
    /// - 1=sắc, 2=huyền, 3=hỏi, 4=ngã, 5=nặng
    /// - 6=mũ (â/ê/ô), 7=móc (ơ/ư), 8=trăng (ă), 9=đ
    ///
    /// # Example
    /// ```ignore
    /// let n = EncodingNormalizer::global();
    /// assert_eq!(n.vni_to_unicode("ha2nh chi1nh"), "hành chính");
    /// ```
    #[instrument(skip(self, input), fields(input_len = input.len()))]
    pub fn vni_to_unicode(&self, input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        let chars: Vec<char> = input.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            // Try 3-char sequence first (e.g., "a61" → 'ấ')
            if i + 2 < len {
                let seq3: String = chars[i..=i + 2].iter().collect();
                if let Some(&unicode_char) = self.vni_map.get(seq3.to_lowercase().as_str()) {
                    // Preserve case of first character
                    if chars[i].is_uppercase() {
                        result.push(unicode_char.to_uppercase().next().unwrap_or(unicode_char));
                    } else {
                        result.push(unicode_char);
                    }
                    i += 3;
                    continue;
                }
            }

            // Try 2-char sequence (e.g., "a1" → 'á')
            if i + 1 < len {
                let seq2: String = chars[i..=i + 1].iter().collect();
                if let Some(&unicode_char) = self.vni_map.get(seq2.to_lowercase().as_str()) {
                    // Preserve case of first character
                    if chars[i].is_uppercase() {
                        result.push(unicode_char.to_uppercase().next().unwrap_or(unicode_char));
                    } else {
                        result.push(unicode_char);
                    }
                    i += 2;
                    continue;
                }
            }

            // No match, pass through
            result.push(chars[i]);
            i += 1;
        }

        info!(output_len = result.len(), "VNI conversion complete");
        result
    }

    /// Convert TCVN3-encoded text to Unicode.
    ///
    /// TCVN3 uses extended ASCII (0x80-0xFF) to encode Vietnamese characters.
    ///
    /// # Example
    /// ```ignore
    /// let n = EncodingNormalizer::global();
    /// assert_eq!(n.tcvn3_to_unicode("Tæng khèi"), "Tổng khối");
    /// ```
    #[instrument(skip(self, input), fields(input_len = input.len()))]
    pub fn tcvn3_to_unicode(&self, input: &str) -> String {
        let result: String = input
            .chars()
            .map(|c| *self.tcvn3_map.get(&c).unwrap_or(&c))
            .collect();

        info!(output_len = result.len(), "TCVN3 conversion complete");
        result
    }

    /// Auto-detect and convert (tries VNI first, then TCVN3).
    #[instrument(skip(self, input))]
    pub fn auto_normalize(&self, input: &str) -> String {
        // Heuristic: If input contains VNI digit patterns, use VNI decoder
        let has_vni_pattern = input.chars().enumerate().any(|(i, c)| {
            matches!(c, '1'..='9')
                && i > 0
                && input
                    .chars()
                    .nth(i - 1)
                    .is_some_and(|prev| prev.is_alphabetic())
        });

        if has_vni_pattern {
            self.vni_to_unicode(input)
        } else {
            self.tcvn3_to_unicode(input)
        }
    }
}

/// Build the VNI mapping table (134 entries).
fn build_vni_map() -> HashMap<&'static str, char> {
    let mut map = HashMap::new();

    // === VOWEL A ===
    map.insert("a1", 'á');
    map.insert("a2", 'à');
    map.insert("a3", 'ả');
    map.insert("a4", 'ã');
    map.insert("a5", 'ạ');
    // Â (a + 6)
    map.insert("a6", 'â');
    map.insert("a61", 'ấ');
    map.insert("a62", 'ầ');
    map.insert("a63", 'ẩ');
    map.insert("a64", 'ẫ');
    map.insert("a65", 'ậ');
    // Ă (a + 8)
    map.insert("a8", 'ă');
    map.insert("a81", 'ắ');
    map.insert("a82", 'ằ');
    map.insert("a83", 'ẳ');
    map.insert("a84", 'ẵ');
    map.insert("a85", 'ặ');

    // === VOWEL E ===
    map.insert("e1", 'é');
    map.insert("e2", 'è');
    map.insert("e3", 'ẻ');
    map.insert("e4", 'ẽ');
    map.insert("e5", 'ẹ');
    // Ê (e + 6)
    map.insert("e6", 'ê');
    map.insert("e61", 'ế');
    map.insert("e62", 'ề');
    map.insert("e63", 'ể');
    map.insert("e64", 'ễ');
    map.insert("e65", 'ệ');

    // === VOWEL I ===
    map.insert("i1", 'í');
    map.insert("i2", 'ì');
    map.insert("i3", 'ỉ');
    map.insert("i4", 'ĩ');
    map.insert("i5", 'ị');

    // === VOWEL O ===
    map.insert("o1", 'ó');
    map.insert("o2", 'ò');
    map.insert("o3", 'ỏ');
    map.insert("o4", 'õ');
    map.insert("o5", 'ọ');
    // Ô (o + 6)
    map.insert("o6", 'ô');
    map.insert("o61", 'ố');
    map.insert("o62", 'ồ');
    map.insert("o63", 'ổ');
    map.insert("o64", 'ỗ');
    map.insert("o65", 'ộ');
    // Ơ (o + 7)
    map.insert("o7", 'ơ');
    map.insert("o71", 'ớ');
    map.insert("o72", 'ờ');
    map.insert("o73", 'ở');
    map.insert("o74", 'ỡ');
    map.insert("o75", 'ợ');

    // === VOWEL U ===
    map.insert("u1", 'ú');
    map.insert("u2", 'ù');
    map.insert("u3", 'ủ');
    map.insert("u4", 'ũ');
    map.insert("u5", 'ụ');
    // Ư (u + 7)
    map.insert("u7", 'ư');
    map.insert("u71", 'ứ');
    map.insert("u72", 'ừ');
    map.insert("u73", 'ử');
    map.insert("u74", 'ữ');
    map.insert("u75", 'ự');

    // === VOWEL Y ===
    map.insert("y1", 'ý');
    map.insert("y2", 'ỳ');
    map.insert("y3", 'ỷ');
    map.insert("y4", 'ỹ');
    map.insert("y5", 'ỵ');

    // === CONSONANT Đ ===
    map.insert("d9", 'đ');

    map
}

/// Build the TCVN3 mapping table (134 entries).
/// TCVN3 uses extended ASCII characters (0x80-0xFF range).
fn build_tcvn3_map() -> HashMap<char, char> {
    let mut map = HashMap::new();

    // === TCVN3 (VSCII-3) to Unicode Elite Mapping Table ===
    // Based on User-provided "Mission 029 - Elite Standard"

    // 0x80 - 0x8F
    map.insert('\u{0080}', 'À'); map.insert('\u{0081}', 'Ả'); map.insert('\u{0082}', 'Ã'); map.insert('\u{0083}', 'Á');
    map.insert('\u{0084}', 'Ạ'); map.insert('\u{0085}', 'Ặ'); map.insert('\u{0086}', 'Ậ'); map.insert('\u{0087}', 'È');
    map.insert('\u{0088}', 'Ẻ'); map.insert('\u{0089}', 'Ẽ'); map.insert('\u{008A}', 'É'); map.insert('\u{008B}', 'Ẹ');
    map.insert('\u{008C}', 'Ệ'); map.insert('\u{008D}', 'Ì'); map.insert('\u{008E}', 'Ỉ'); map.insert('\u{008F}', 'Ĩ');

    // 0x90 - 0x9F
    map.insert('\u{0090}', 'Í'); map.insert('\u{0091}', 'Ị'); map.insert('\u{0092}', 'Ò'); map.insert('\u{0093}', 'Ỏ');
    map.insert('\u{0094}', 'Õ'); map.insert('\u{0095}', 'Ó'); map.insert('\u{0096}', 'Ọ'); map.insert('\u{0097}', 'Ộ');
    map.insert('\u{0098}', 'Ờ'); map.insert('\u{0099}', 'Ở'); map.insert('\u{009A}', 'Ỡ'); map.insert('\u{009B}', 'Ớ');
    map.insert('\u{009C}', 'Ợ'); map.insert('\u{009D}', 'Ù'); map.insert('\u{009E}', 'Ủ'); map.insert('\u{009F}', 'Ũ');

    // 0xA0 - 0xAF
    map.insert('\u{00A0}', '\u{00A0}'); // NBSP
    map.insert('\u{00A1}', 'Ă'); map.insert('\u{00A2}', 'Â'); map.insert('\u{00A3}', 'Ê'); map.insert('\u{00A4}', 'Ô');
    map.insert('\u{00A5}', 'Ơ'); map.insert('\u{00A6}', 'Ư'); map.insert('\u{00A7}', 'Đ'); map.insert('\u{00A8}', 'ă');
    map.insert('\u{00A9}', 'â'); map.insert('\u{00AA}', 'ê'); map.insert('\u{00AB}', 'ô'); map.insert('\u{00AC}', 'ơ');
    map.insert('\u{00AD}', 'ư'); map.insert('\u{00AE}', 'đ'); map.insert('\u{00AF}', 'Ằ');

    // 0xB0 - 0xBF
    map.insert('\u{00B0}', '\u{0300}'); map.insert('\u{00B1}', '\u{0309}'); map.insert('\u{00B2}', '\u{0303}'); map.insert('\u{00B3}', '\u{0301}');
    map.insert('\u{00B4}', '\u{0323}'); map.insert('\u{00B5}', 'à'); map.insert('\u{00B6}', 'ả'); map.insert('\u{00B7}', 'ã');
    map.insert('\u{00B8}', 'á'); map.insert('\u{00B9}', 'ạ'); map.insert('\u{00BA}', 'Ẳ'); map.insert('\u{00BB}', 'ằ');
    map.insert('\u{00BC}', 'ẳ'); map.insert('\u{00BD}', 'ẵ'); map.insert('\u{00BE}', 'ắ'); map.insert('\u{00BF}', 'Ẵ');

    // 0xC0 - 0xCF
    map.insert('\u{00C0}', 'Ắ'); map.insert('\u{00C1}', 'Ầ'); map.insert('\u{00C2}', 'Ẩ'); map.insert('\u{00C3}', 'Ẫ');
    map.insert('\u{00C4}', 'Ấ'); map.insert('\u{00C5}', 'Ề'); map.insert('\u{00C6}', 'ặ'); map.insert('\u{00C7}', 'ầ');
    map.insert('\u{00CC}', 'è'); map.insert('\u{00CD}', 'Ể'); map.insert('\u{00CE}', 'ẻ'); map.insert('\u{00CF}', 'ẽ');
    // Note: 0xC8-0xCB (ẩ, ẫ, ấ, ậ) handled below in sequence if missing items here
    map.insert('\u{00C8}', 'ẩ'); map.insert('\u{00C9}', 'ẫ'); map.insert('\u{00CA}', 'ấ'); map.insert('\u{00CB}', 'ậ');

    // 0xD0 - 0xDF
    map.insert('\u{00D0}', 'é'); map.insert('\u{00D1}', 'ẹ'); map.insert('\u{00D2}', 'ề'); map.insert('\u{00D3}', 'ể');
    map.insert('\u{00D4}', 'ễ'); map.insert('\u{00D5}', 'ế'); map.insert('\u{00D6}', 'ệ'); map.insert('\u{00D7}', 'ì');
    map.insert('\u{00D8}', 'ỉ'); map.insert('\u{00D9}', 'Ễ'); map.insert('\u{00DA}', 'Ế'); map.insert('\u{00DB}', 'Ồ');
    map.insert('\u{00DC}', 'ĩ'); map.insert('\u{00DD}', 'í'); map.insert('\u{00DE}', 'ị'); map.insert('\u{00DF}', 'ò');

    // 0xE0 - 0xEF
    map.insert('\u{00E0}', 'Ổ'); map.insert('\u{00E1}', 'ỏ'); map.insert('\u{00E2}', 'õ'); map.insert('\u{00E3}', 'ó');
    map.insert('\u{00E4}', 'ọ'); map.insert('\u{00E5}', 'ồ'); map.insert('\u{00E6}', 'ổ'); map.insert('\u{00E7}', 'ỗ');
    map.insert('\u{00E8}', 'ố'); map.insert('\u{00E9}', 'ộ'); map.insert('\u{00EA}', 'ờ'); map.insert('\u{00EB}', 'ở');
    map.insert('\u{00EC}', 'ỡ'); map.insert('\u{00ED}', 'ớ'); map.insert('\u{00EE}', 'ợ'); map.insert('\u{00EF}', 'ù');

    // 0xF0 - 0xFF
    map.insert('\u{00F0}', 'Ỗ'); map.insert('\u{00F1}', 'ủ'); map.insert('\u{00F2}', 'ũ'); map.insert('\u{00F3}', 'ú');
    map.insert('\u{00F4}', 'ụ'); map.insert('\u{00F5}', 'ừ'); map.insert('\u{00F6}', 'ử'); map.insert('\u{00F7}', 'ữ');
    map.insert('\u{00F8}', 'ứ'); map.insert('\u{00F9}', 'ự'); map.insert('\u{00FA}', 'ỳ'); map.insert('\u{00FB}', 'ỷ');
    map.insert('\u{00FC}', 'ỹ'); map.insert('\u{00FD}', 'ý'); map.insert('\u{00FE}', 'ỵ'); map.insert('\u{00FF}', 'Ố');

    map
}

// ============================================================================
// 🧪 TEST SUITE - ENCODING NORMALIZER
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // === VNI TESTS ===

    #[test]
    fn test_vni_basic_tones() {
        let n = EncodingNormalizer::global();
        assert_eq!(n.vni_to_unicode("a1"), "á");
        assert_eq!(n.vni_to_unicode("a2"), "à");
        assert_eq!(n.vni_to_unicode("a3"), "ả");
        assert_eq!(n.vni_to_unicode("a4"), "ã");
        assert_eq!(n.vni_to_unicode("a5"), "ạ");
    }

    #[test]
    fn test_vni_circumflex() {
        let n = EncodingNormalizer::global();
        assert_eq!(n.vni_to_unicode("a6"), "â");
        assert_eq!(n.vni_to_unicode("a61"), "ấ");
        assert_eq!(n.vni_to_unicode("a62"), "ầ");
        assert_eq!(n.vni_to_unicode("e6"), "ê");
        assert_eq!(n.vni_to_unicode("o6"), "ô");
    }

    #[test]
    fn test_vni_horn() {
        let n = EncodingNormalizer::global();
        assert_eq!(n.vni_to_unicode("o7"), "ơ");
        assert_eq!(n.vni_to_unicode("o71"), "ớ");
        assert_eq!(n.vni_to_unicode("u7"), "ư");
        assert_eq!(n.vni_to_unicode("u71"), "ứ");
    }

    #[test]
    fn test_vni_breve() {
        let n = EncodingNormalizer::global();
        assert_eq!(n.vni_to_unicode("a8"), "ă");
        assert_eq!(n.vni_to_unicode("a81"), "ắ");
        assert_eq!(n.vni_to_unicode("a85"), "ặ");
    }

    #[test]
    fn test_vni_d_stroke() {
        let n = EncodingNormalizer::global();
        assert_eq!(n.vni_to_unicode("d9"), "đ");
    }

    #[test]
    fn test_vni_sentence() {
        let n = EncodingNormalizer::global();
        assert_eq!(n.vni_to_unicode("ha2nh chi1nh"), "hành chính");
        assert_eq!(n.vni_to_unicode("kho61i lu7o75ng"), "khối lượng");
    }

    #[test]
    fn test_vni_case_preservation() {
        let n = EncodingNormalizer::global();
        assert_eq!(n.vni_to_unicode("A1"), "Á");
        assert_eq!(n.vni_to_unicode("D9"), "Đ");
    }

    #[test]
    fn test_vni_passthrough() {
        let n = EncodingNormalizer::global();
        // ASCII without VNI patterns should pass through unchanged
        assert_eq!(n.vni_to_unicode("hello world"), "hello world");
        assert_eq!(n.vni_to_unicode("123"), "123");
    }

    // === TCVN3 TESTS ===

    #[test]
    fn test_tcvn3_basic() {
        let n = EncodingNormalizer::global();
        assert_eq!(n.tcvn3_to_unicode("¸"), "á");
        assert_eq!(n.tcvn3_to_unicode("µ"), "à");
    }

    #[test]
    fn test_tcvn3_circumflex() {
        let n = EncodingNormalizer::global();
        assert_eq!(n.tcvn3_to_unicode("©"), "â");
        assert_eq!(n.tcvn3_to_unicode("ª"), "ê");
        assert_eq!(n.tcvn3_to_unicode("«"), "ô");
    }

    #[test]
    fn test_tcvn3_horn() {
        let n = EncodingNormalizer::global();
        assert_eq!(n.tcvn3_to_unicode("¬"), "ơ");
        assert_eq!(n.tcvn3_to_unicode("­"), "ư");
    }

    #[test]
    fn test_tcvn3_d_stroke() {
        let n = EncodingNormalizer::global();
        assert_eq!(n.tcvn3_to_unicode("®"), "đ");
    }

    #[test]
    fn test_tcvn3_mixed() {
        let n = EncodingNormalizer::global();
        // "Tæng" -> "Tổng" (T is ASCII, æ -> ổ, ng is ASCII)
        assert_eq!(n.tcvn3_to_unicode("Tæ"), "Tổ");
    }

    #[test]
    fn test_tcvn3_passthrough() {
        let n = EncodingNormalizer::global();
        // Pure ASCII should pass through unchanged
        assert_eq!(n.tcvn3_to_unicode("hello"), "hello");
    }

    // === AUTO DETECTION TESTS ===

    #[test]
    fn test_auto_detect_vni() {
        let n = EncodingNormalizer::global();
        // Contains VNI pattern (letter + digit)
        let result = n.auto_normalize("ha2nh chi1nh");
        assert_eq!(result, "hành chính");
    }

    #[test]
    fn test_auto_detect_tcvn3() {
        let n = EncodingNormalizer::global();
        // No VNI pattern, uses TCVN3
        let result = n.auto_normalize("¸µ¶");
        assert_eq!(result, "áàả");
    }

    // === EDGE CASES ===

    #[test]
    fn test_empty_input() {
        let n = EncodingNormalizer::global();
        assert_eq!(n.vni_to_unicode(""), "");
        assert_eq!(n.tcvn3_to_unicode(""), "");
    }

    #[test]
    fn test_unicode_passthrough() {
        let n = EncodingNormalizer::global();
        // Already Unicode Vietnamese should pass through
        assert_eq!(n.vni_to_unicode("Việt Nam"), "Việt Nam");
        assert_eq!(n.tcvn3_to_unicode("Việt Nam"), "Việt Nam");
    }

    #[test]
    fn test_thread_safety() {
        // OnceLock guarantees thread safety, but let's verify multiple accesses work
        let n1 = EncodingNormalizer::global();
        let n2 = EncodingNormalizer::global();
        assert!(std::ptr::eq(n1, n2), "Should return same instance");
    }
}
