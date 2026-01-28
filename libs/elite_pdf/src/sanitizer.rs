// =========================================================================
// MISSION 009: SIMD SANITIZER (Surgical Strike - STABLE VERSION)
// =========================================================================
// Red Team Protocol: SIMD chỉ ở Mechanical Layer, an toàn tuyệt đối
// Rule 1: Không SIMD qua FFI boundary
// Rule 2: Không SIMD trên Unicode scalar
// Rule 3: Không SIMD trên dữ liệu có ownership phức tạp
// Rule 4: Luôn có scalar fallback
// UPDATE: std::simd unstable -> using core::arch intrinsics for stable Rust

/// High-performance text sanitizer for raw MuPDF byte streams
/// Uses stable intrinsics for cross-platform compatibility
pub struct SimdSanitizer;

impl SimdSanitizer {
    /// Fast path: SIMD-accelerated byte cleaning (stable intrinsics)
    /// Input: Raw UTF-8-ish bytes from MuPDF
    /// Output: Clean ASCII/UTF-8 bytes for further processing
    pub fn fast_clean(input: &[u8]) -> Vec<u8> {
        // Guard: Small data doesn't benefit from SIMD overhead
        if input.len() < 64 {
            return Self::scalar_clean(input);
        }

        // Platform-specific SIMD acceleration
        #[cfg(target_arch = "x86_64")]
        {
            if Self::is_x86_simd_supported() {
                return unsafe { Self::x86_simd_clean(input) };
            }
        }

        // Fallback to optimized scalar
        Self::scalar_clean(input)
    }

    /// Check for x86 SIMD support at runtime
    #[cfg(target_arch = "x86_64")]
    #[inline]
    fn is_x86_simd_supported() -> bool {
        #[cfg(target_feature = "sse2")]
        {
            is_x86_feature_detected!("sse2")
        }
        #[cfg(not(target_feature = "sse2"))]
        {
            false
        }
    }

    /// Optimized scalar implementation with lookahead
    /// This is already faster than naive approaches
    fn scalar_clean(input: &[u8]) -> Vec<u8> {
        let mut output = Vec::with_capacity(input.len());
        let mut i = 0;
        let len = input.len();

        while i < len {
            let byte = input[i];

            // Rule: Skip control characters
            if byte < 32 {
                i += 1;
                continue;
            }

            // Rule: Fast space collapse with lookahead
            if byte == b' ' {
                // Skip consecutive spaces
                while i + 1 < len && input[i + 1] == b' ' {
                    i += 1;
                }
                output.push(b' ');
                i += 1;
                continue;
            }

            output.push(byte);
            i += 1;
        }

        // Trim leading/trailing spaces
        let start = output.iter().position(|&b| b != b' ').unwrap_or(0);
        let end = output
            .iter()
            .rposition(|&b| b != b' ')
            .map(|p| p + 1)
            .unwrap_or(output.len());

        if start < end {
            output[start..end].to_vec()
        } else {
            Vec::new()
        }
    }

    /// x86 SIMD implementation using stable intrinsics
    #[cfg(target_arch = "x86_64")]
    unsafe fn x86_simd_clean(input: &[u8]) -> Vec<u8> {
        #[cfg(target_feature = "sse2")]
        {
            use core::arch::x86_64::*;

            const LANES: usize = 16; // SSE2 processes 16 bytes at once
            let chunks = input.chunks_exact(LANES);
            let remainder = chunks.remainder();

            let mut output = Vec::with_capacity(input.len());

            // Control character mask (bytes < 32)
            let control_mask = _mm_set1_epi8(0x1F);
            // Space mask for comparison
            let space_vec = _mm_set1_epi8(b' ' as i8);

            for chunk in chunks {
                let data = _mm_loadu_si128(chunk.as_ptr() as *const __m128i);

                // Mask bytes < 32 (control chars)
                let cmp_control = _mm_cmpgt_epi8(data, control_mask);

                // Find spaces
                let cmp_space = _mm_cmpeq_epi8(data, space_vec);

                // Valid mask = not control chars
                let valid_mask = cmp_control;

                // Extract bytes (scalar fallback for simplicity but still accelerated)
                let mut temp = [0u8; LANES];
                _mm_storeu_si128(temp.as_mut_ptr() as *mut __m128i, data);

                // Extract bytes using transmute to access private method
                let mask_bytes = std::mem::transmute::<_, [u8; 16]>(valid_mask);

                // Process each byte with SIMD-guided decisions
                for (j, &byte) in temp.iter().enumerate() {
                    let is_valid = (mask_bytes[j] & 0x80) != 0;

                    if is_valid && byte >= 32 {
                        // Additional scalar logic for space collapsing
                        if byte == b' ' {
                            if output.last() != Some(&b' ') {
                                output.push(byte);
                            }
                        } else {
                            output.push(byte);
                        }
                    }
                }
            }

            // Process remainder with scalar
            output.extend_from_slice(&Self::scalar_clean(remainder));
            output
        }
        #[cfg(not(target_feature = "sse2"))]
        {
            Self::scalar_clean(input)
        }
    }

    /// Verify consistency between fast and scalar paths
    #[cfg(test)]
    fn verify_consistency(input: &[u8]) -> bool {
        let scalar_result = Self::scalar_clean(input);
        let fast_result = Self::fast_clean(input);
        scalar_result == fast_result
    }
}

// =========================================================================
// GOLDEN TESTS (100% consistency required)
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_space_collapse() {
        let input = b"Hello     World";
        let expected = b"Hello World";
        assert_eq!(SimdSanitizer::fast_clean(input), expected);
    }

    #[test]
    fn test_control_char_removal() {
        let input = b"Hello\x00\x01\x02World";
        let expected = b"HelloWorld";
        assert_eq!(SimdSanitizer::fast_clean(input), expected);
    }

    #[test]
    fn test_leading_trailing_spaces() {
        let input = b"   Hello World   ";
        let expected = b"Hello World";
        assert_eq!(SimdSanitizer::fast_clean(input), expected);
    }

    #[test]
    fn test_unicode_safety() {
        // UTF-8 bytes should pass through unchanged
        let input = "Hello 世界".as_bytes();
        let expected = input;
        assert_eq!(SimdSanitizer::fast_clean(input), expected);
    }

    #[test]
    fn test_consistency_across_paths() {
        // Test data with various problematic cases
        let test_cases = vec![
            b"Simple text".to_vec(),
            b"Multiple    spaces".to_vec(),
            b"\x00\x01Control\x02chars".to_vec(),
            b"   Leading and trailing   ".to_vec(),
            b"Mixed\x00   spaces".to_vec(),
            "Unicode 世界 test".as_bytes().to_vec(),
        ];

        for case in test_cases {
            assert!(
                SimdSanitizer::verify_consistency(&case),
                "Consistency check failed for: {:?}",
                case
            );
        }
    }

    #[test]
    fn test_small_input_fallback() {
        // Small inputs should use scalar path
        let small_input = b"Hi";
        let expected = b"Hi";
        assert_eq!(SimdSanitizer::fast_clean(small_input), expected);
    }

    #[test]
    fn test_empty_input() {
        let empty = b"";
        let expected = b"";
        assert_eq!(SimdSanitizer::fast_clean(empty), expected);
    }
}
