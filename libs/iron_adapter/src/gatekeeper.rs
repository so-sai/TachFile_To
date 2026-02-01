use iron_table::contract::EncodingStatus;

pub struct EncodingGatekeeper;

const MOJIBAKE_SCARS: &[&str] = &[
    "\u{251C}\u{00AC}", // ├¬ (ê)
    "\u{251C}\u{2524}", // ├┤ (ô)
    "\u{251C}\u{2502}", // ├│ (ó)
    "\u{251C}\u{00E1}", // ├á (à)
    "\u{251C}\u{2551}", // ├║ (ú)
    "\u{251C}\u{00ED}", // ├¡ (í)
    "\u{251C}\u{00AE}", // ├® (é)
    "\u{251C}\u{00C2}", // ├Â (Â)
];

impl EncodingGatekeeper {
    /// Scans a string for character integrity.
    /// Returns (Status, Evidence)
    /// 
    /// # LAW-07 Compliance
    /// Gatekeeper DETECTS ONLY. It does NOT repair.
    /// Repair is handled by RepairEngine in the Human Repair Loop.
    pub fn scan(input: &str) -> (EncodingStatus, Option<String>) {
        if input.is_empty() {
            return (EncodingStatus::Clean, None);
        }

        // 1. Fixed "Scars" Detection (Highest Accuracy)
        for scar in MOJIBAKE_SCARS {
            if input.contains(scar) {
                return (EncodingStatus::Invalid, Some(format!("Found Mojibake scar: '{}'", scar)));
            }
        }

        // 2. Box-Drawing Density Heuristic
        // CP437 interpretation of UTF-8 VN text often results in box-drawing characters
        let box_drawing_chars = ['├', '┤', '┐', '└', '┴', '┬', '┼', '│', '─'];
        let mut box_chars_in_words = 0;
        let char_count = input.chars().count();
        
        let chars: Vec<char> = input.chars().collect();
        for i in 0..chars.len() {
            if box_drawing_chars.contains(&chars[i]) {
                // Heuristic: Is it adjacent to alphabetic characters?
                let adjacent_to_alpha = (i > 0 && chars[i-1].is_alphabetic()) 
                    || (i < chars.len() - 1 && chars[i+1].is_alphabetic());
                
                if adjacent_to_alpha {
                    box_chars_in_words += 1;
                }
            }
        }

        let density = box_chars_in_words as f32 / char_count as f32;

        if density > 0.1 {
            (EncodingStatus::Invalid, Some(format!("High density box-drawing characters ({:.2})", density)))
        } else if box_chars_in_words > 0 {
            (EncodingStatus::Suspicious, Some(format!("Found {} suspicious box characters", box_chars_in_words)))
        } else {
            (EncodingStatus::Clean, None)
        }
    }

    /// Convenience helper for quick checks.
    pub fn is_mojibake(input: &str) -> bool {
        matches!(Self::scan(input).0, EncodingStatus::Invalid)
    }

    // NOTE: try_repair() intentionally NOT implemented here.
    // Per LAW-07 (Fail Safe): "When in doubt, BLOCK, don't FIX"
    // Repair is Human-Gated via RepairEngine.
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gatekeeper_clean() {
        assert_eq!(EncodingGatekeeper::scan("Bê tông móng").0, EncodingStatus::Clean);
        assert_eq!(EncodingGatekeeper::scan("1.250.000").0, EncodingStatus::Clean);
        assert_eq!(EncodingGatekeeper::scan("│ Khối lượng │").0, EncodingStatus::Clean); // Literal box drawing (not word-adjacent)
    }

    #[test]
    fn test_gatekeeper_mojibake() {
        // The exact mojibake string from the test failure
        let corrupted = "B├¬ t├┤ng m├│ng"; 
        let (status, evidence) = EncodingGatekeeper::scan(corrupted);
        assert_eq!(status, EncodingStatus::Invalid);
        assert!(evidence.unwrap().contains("Found Mojibake scar"));
        
        let partial_corrupt = "Text with suspicious├character";
        assert_eq!(EncodingGatekeeper::scan(partial_corrupt).0, EncodingStatus::Suspicious);
    }
}
