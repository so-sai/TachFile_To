// =========================================================================
// MISSION 010.5: THE SHIELD - TDD Test Suite for Semantic Engine
// =========================================================================
// Bảo vệ Hiến pháp SEMANTIC_CONTRACT_V1.md bằng các Test Case thép
// Mục tiêu: Không một dòng code semantic được phép chạy nếu không qua được đây

use crate::semantic::encoding::{EncodingMapper, VnEncoding};
use crate::semantic::engine::SemanticEngine;
use proptest::prelude::*;

// =========================================================================
// 1. GREEDY VNI DECODER TESTS - LỖI THÉP CHO DI SẢN
// =========================================================================

#[cfg(test)]
mod vni_decoder_tests {
    use super::*;

    #[test]
    fn test_vni_standard_case() {
        // Case 1: u + 7 + 8 = ướ (Tiếng Việt chuẩn di sản)
        let input = "u78";
        let encoding = VnEncoding::Vni;

        let mut decoder = crate::semantic::encoding::VniDecoder::new();
        let result = decoder.decode(input.to_string());

        assert_eq!(result, "ướ");
    }

    #[test]
    fn test_vni_mixed_case() {
        // Case 2: o + 7 + 2 = ờ (Xử lý dấu thanh sau dấu mũ)
        let input = "o72";
        let encoding = VnEncoding::Vni;

        let mut decoder = crate::semantic::encoding::VniDecoder::new();
        let result = decoder.decode(input.to_string());

        assert_eq!(result, "ờ");
    }

    #[test]
    fn test_vni_resilience_case() {
        // Case 3: a + 8 + [RÁC] -> Đảm bảo không crash
        let input = "a8[RÁC]";
        let encoding = VnEncoding::Vni;

        let mut decoder = crate::semantic::encoding::VniDecoder::new();
        let result = decoder.decode(input.to_string());

        assert_eq!(result, "ă[RÁC]");
    }

    // Property-based testing cho VNI decoder
    proptest! {
        #[test]
        fn vni_decoder_never_crashes(input in "[a-zA-Z0-9\\[\\]]*") {
            let encoding = VnEncoding::Vni;
            let mut decoder = crate::semantic::encoding::VniDecoder::new();

            // Đảm bảo không panic với bất kỳ input nào
            let _result = decoder.decode(input);
        }
    }
}

// =========================================================================
// 2. GEOMETRIC LINE MERGING TESTS - HÌNH HỌC LÀ LUẬT
// =========================================================================

#[cfg(test)]
mod geometric_merging_tests {
    use super::*;

    #[test]
    fn test_visual_connection_rule() {
        // Kiểm tra quy tắc: Khoảng cách < 1.5 * line_height + Alignment
        let prev_bbox = crate::mupdf_text::FzRect {
            x0: 50.0,
            y0: 100.0,
            x1: 300.0,
            y1: 115.0,
        };
        let curr_bbox = crate::mupdf_text::FzRect {
            x0: 50.0,
            y0: 117.0,
            x1: 280.0,
            y1: 132.0,
        };
        let line_height_mode = 15.0; // Mode của line height trong trang

        // Vertical proximity: 117.0 - 115.0 = 2.0 < 1.5 * 15.0 = 22.5 ✅
        // Horizontal alignment: abs(50.0 - 50.0) = 0.0 < tolerance ✅

        let analyzer = crate::semantic::spatial::GeometricAnalyzer::new();
        let is_connected = analyzer.is_visually_connected(&prev_bbox, &curr_bbox, line_height_mode);

        assert!(
            is_connected,
            "Should be connected - vertical gap and alignment satisfied"
        );
    }

    #[test]
    fn test_heading_isolation_rule() {
        // Kiểm tra quy tắc "Không gian thở" của Heading
        let line_height_mode = 12.0;
        let gap_before_bold = 25.0; // > 1.8 * 12.0 = 21.6
        let gap_small = 15.0; // < 21.6

        assert!(crate::semantic::engine::is_structural_heading(
            gap_before_bold,
            line_height_mode
        ));
        assert!(!crate::semantic::engine::is_structural_heading(
            gap_small,
            line_height_mode
        ));
    }

    #[test]
    fn test_union_bbox_expansion() {
        // Kiểm tra BBox mở rộng khi gộp dòng
        let mut paragraph = crate::semantic::blocks::ParagraphBlock::new();

        let line1_bbox = crate::mupdf_text::FzRect {
            x0: 50.0,
            y0: 100.0,
            x1: 300.0,
            y1: 115.0,
        };
        let line2_bbox = crate::mupdf_text::FzRect {
            x0: 50.0,
            y0: 117.0,
            x1: 280.0,
            y1: 132.0,
        };

        paragraph.add_line("Dòng đầu tiên", line1_bbox);
        paragraph.add_line("Dòng thứ hai", line2_bbox);

        let union_bbox = paragraph.get_union_bbox();
        let expected = crate::mupdf_text::FzRect {
            x0: 50.0,
            y0: 100.0,
            x1: 300.0,
            y1: 132.0,
        };

        assert_eq!(union_bbox.x0, expected.x0);
        assert_eq!(union_bbox.y0, expected.y0);
        assert_eq!(union_bbox.x1, expected.x1);
        assert_eq!(union_bbox.y1, expected.y1);
    }
}

// =========================================================================
// 3. SOVEREIGN TRUTH INTEGRATION TESTS - BẢNG CHỨNG KẾT QUẢ
// =========================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_golden_standard_pipeline() {
        // Test với PDF "vàng" (golden PDF)
        let test_pdf = Path::new("test_files/golden_standard.pdf");

        if test_pdf.exists() {
            // Chạy toàn bộ pipeline
            let doc = crate::EliteDocument::new(test_pdf.to_str().unwrap()).unwrap();
            let markdown = doc.extract_page_markdown(0).unwrap();

            // Kiểm tra Contract V1:
            // 1. Deterministic: Chạy lại 100 lần ra cùng kết quả
            for _ in 0..100 {
                let again = doc.extract_page_markdown(0).unwrap();
                assert_eq!(markdown, again);
            }

            // 2. Metadata: Mỗi block phải có BBox
            assert!(markdown.contains("<!-- bbox:"));

            // 3. Block structure: Không được có dòng trôi lẻ
            let lines: Vec<&str> = markdown.lines().collect();
            for line in lines {
                if line.trim().len() > 0 && !line.starts_with('#') && !line.starts_with('-') {
                    // Dữ liệu text thường phải nằm trong paragraph hoàn chỉnh
                    assert!(
                        !line.ends_without_whitespace(),
                        "Line should not be orphaned"
                    );
                }
            }
        }
    }

    #[test]
    fn test_alignment_persistence() {
        // Property test cho Alignment Persistence với jitter ngẫu nhiên
        proptest! {
            #[test]
            fn alignment_persists_with_jitter(
                base_x in 50.0..200.0f32,
                jitter in -2.0..2.0f32
            ) {
                let col1 = base_x;
                let col2 = base_x + 100.0;

                // Tạo các dòng có jitter nhỏ
                let line1 = crate::semantic::spatial::ColumnInfo { x: col1 + jitter };
                let line2 = crate::semantic::spatial::ColumnInfo { x: col2 + jitter };

                let validator = crate::semantic::spatial::TableValidator::new();
                validator.add_line_columns(vec![line1, line2]);

                // Với jitter nhỏ, alignment vẫn phải được bảo toàn
                assert!(validator.confidence() > 0.8, "Alignment should persist with small jitter");
            }
        }
    }
}

// =========================================================================
// 4. CONTRACT VIOLATION DETECTION - TỰ ĐỘNG PHÁT HIỆN VI PHẠM
// =========================================================================

#[cfg(test)]
mod contract_violation_tests {
    use super::*;

    #[test]
    fn test_detects_heading_without_breathing_room() {
        // Phát hiện Heading vi phạm quy tắc "không gian thở"
        let text = "**Đây không phải heading**";
        let line_height_mode = 12.0;
        let gap_before = 15.0; // < 1.8 * 12.0 = 21.6

        let result = crate::semantic::engine::classify_structural_element(
            text,
            gap_before,
            line_height_mode,
        );

        // Phải được hạ cấp xuống Bold Paragraph
        match result {
            crate::semantic::engine::DocumentState::BoldParagraph => {} // ✅ Expected
            crate::semantic::engine::DocumentState::Heading(_) => {
                panic!("Should not be heading without breathing room")
            }
            _ => panic!("Unexpected classification"),
        }
    }

    #[test]
    fn test_enforces_metadata_per_block() {
        // Đảm bảo metadata gắn vào Block, không phải Line
        let mut engine = SemanticEngine::new_test();

        // Simulate 2 dòng cùng paragraph
        engine.process_line_for_test("Dòng đầu tiên", 100.0);
        engine.process_line_for_test("Dòng thứ hai", 115.0);

        let output = engine.get_markdown();

        // Chỉ nên có 1 bbox comment cho cả paragraph
        let bbox_count = output.matches("<!-- bbox:").count();
        assert_eq!(
            bbox_count, 1,
            "Should have exactly 1 bbox for entire paragraph"
        );
    }
}

// =========================================================================
// 5. PERFORMANCE REGRESSION TESTS - ĐẢM BẢO TỐC ĐỘ
// =========================================================================

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_simd_sanitizer_performance() {
        // Kiểm tra SIMD sanitizer không chậm hơn scalar
        let large_text = "Hello     World     ".repeat(10000);
        let bytes = large_text.as_bytes();

        // Test scalar path
        let start = Instant::now();
        for _ in 0..1000 {
            let _scalar = crate::sanitizer::SimdSanitizer::scalar_clean(bytes);
        }
        let scalar_time = start.elapsed();

        // Test SIMD path (nếu có)
        let start = Instant::now();
        for _ in 0..1000 {
            let _simd = crate::sanitizer::SimdSanitizer::fast_clean(bytes);
        }
        let simd_time = start.elapsed();

        // SIMD phải nhanh hơn hoặc bằng scalar
        if cfg!(target_arch = "x86_64") {
            assert!(
                simd_time <= scalar_time * 110 / 100,
                "SIMD should not be significantly slower than scalar"
            );
        }
    }

    #[test]
    fn test_semantic_engine_single_pass() {
        // Đảm bảo engine chỉ single pass, không backtracking
        let large_input = create_test_document(1000); // 1000 lines

        let start = Instant::now();
        let _result = crate::semantic::engine::process_large_document(&large_input);
        let processing_time = start.elapsed();

        // Single pass nên xử lý 1000 lines trong < 100ms
        assert!(
            processing_time.as_millis() < 100,
            "Single pass engine should process 1000 lines quickly"
        );
    }
}

fn create_test_document(line_count: usize) -> Vec<String> {
    (0..line_count)
        .map(|i| format!("Dòng văn bản số {}", i))
        .collect()
}
