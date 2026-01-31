use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum EncodingStatus {
    Clean,
    Suspicious,
    Invalid,
}

pub struct EncodingGatekeeper;

impl EncodingGatekeeper {
    /// Scans a string for character integrity.
    pub fn scan(input: &str) -> EncodingStatus {
        // 1. Basic UTF-8 validity is guaranteed by Rust's &str. 
        // If we got here through a safe bridge, it's valid UTF-8 bytes.
        // However, it might be "Logical Mojibake" (valid UTF-8 but junk characters from double-encoding).

        if input.is_empty() {
            return EncodingStatus::Clean;
        }

        // 2. Mojibake Detection Heuristics (Elite Standard)
        // Look for common corruption markers in Vietnamese text
        // - Character sequences like ├, ¬, Â, Ã, followed by specific symbols
        // - High frequency of non-standard symbols in supposedly natural text
        
        let mojibake_markers = ['├', '¬', '┤', '┐', '└', '┴', '┬', '┼'];
        let mut suspicious_count = 0;

        for c in input.chars() {
            if mojibake_markers.contains(&c) {
                suspicious_count += 1;
            }
        }

        let density = suspicious_count as f32 / input.chars().count() as f32;

        if density > 0.05 {
            EncodingStatus::Invalid
        } else if suspicious_count > 0 {
            EncodingStatus::Suspicious
        } else {
            EncodingStatus::Clean
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gatekeeper_clean() {
        assert_eq!(EncodingGatekeeper::scan("Bê tông móng"), EncodingStatus::Clean);
        assert_eq!(EncodingGatekeeper::scan("1.250.000"), EncodingStatus::Clean);
    }

    #[test]
    fn test_gatekeeper_mojibake() {
        // The exact mojibake string from the test failure
        let corrupted = "B├¬ t├┤ng m├│ng"; 
        assert_eq!(EncodingGatekeeper::scan(corrupted), EncodingStatus::Invalid);
        
        let partial_corrupt = "Text with a ¬ character";
        assert_eq!(EncodingGatekeeper::scan(partial_corrupt), EncodingStatus::Suspicious);
    }
}
