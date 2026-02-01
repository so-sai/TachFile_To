use iron_table::contract::*;
use iron_table::normalizer::*;
use std::path::PathBuf;

#[test]
fn test_reject_ambiguous_structure() {
    // This test simulates a TableTruth constructed with a violation
    // Ideally, we test via the validate_contract method
    
    let schema = TableSchema {
        columns: vec![
            ColumnDef { name: "stt".to_string(), dtype: DataType::Int, unit: None, nullable: false, is_critical: false }
        ],
        row_count: 100,
        col_count: 5 // Mismatch with columns len (1)
    };
    
    let rows = vec![]; // Empty rows for this test case
    
    let table = TableTruth {
        table_id: "test".to_string(),
        source_file: PathBuf::from("test.pdf"),
        source_page: 1,
        schema,
        rows,
        extraction_meta: ExtractionMeta { tool_version: "v1".to_string(), timestamp: "now".to_string(), confidence_score: 0.9 },
        bbox: BoundingBox { x: 0.0, y: 0.0, width: 100.0, height: 100.0, page: 1 }
    };

    let result = table.validate_contract();
    assert!(result.is_err());
    
    match result {
        Err(TableRejection::ContractViolation(msg)) => {
            assert!(msg.contains("Schema column count"));
        },
        _ => panic!("Expected ContractViolation"),
    }
}

#[test]
fn test_reject_low_confidence() {
    let schema = TableSchema {
        columns: vec![
            ColumnDef { name: "val".to_string(), dtype: DataType::Float64, unit: None, nullable: false, is_critical: false }
        ],
        row_count: 2, // Satisfy "row_count < 2" check
        col_count: 1
    };
    
    let rows = vec![
        TableRow {
            row_idx: 0,
            cells: vec![
                TableCell {
                    global_id: "test_0_0".to_string(),
                    row_idx: 0, col_idx: 0, value: CellValue::Float(1.0),
                    bbox: BoundingBox { x: 0.0, y: 0.0, width: 10.0, height: 10.0, page: 1 },
                    confidence: 0.9, source_text: "1.0".to_string(),
                    encoding_status: EncodingStatus::Clean, encoding_evidence: None
                }
            ]
        },
        TableRow {
            row_idx: 1,
            cells: vec![
                TableCell {
                    global_id: "test_1_0".to_string(),
                    row_idx: 1, col_idx: 0, value: CellValue::Float(1.0),
                    bbox: BoundingBox { x: 0.0, y: 10.0, width: 10.0, height: 10.0, page: 1 },
                    confidence: 0.6, // Low confidence trigger
                    source_text: "1.0".to_string(),
                    encoding_status: EncodingStatus::Clean, encoding_evidence: None
                }
            ]
        }
    ];

     let table = TableTruth {
        table_id: "test".to_string(),
        source_file: PathBuf::from("test.pdf"),
        source_page: 1,
        schema,
        rows,
        extraction_meta: ExtractionMeta { tool_version: "v1".to_string(), timestamp: "now".to_string(), confidence_score: 0.9 },
        bbox: BoundingBox { x: 0.0, y: 0.0, width: 100.0, height: 100.0, page: 1 }
    };

    let result = table.validate_contract();
    assert!(result.is_err());
     match result {
        Err(TableRejection::LowConfidence(msg)) => {
            assert!(msg.contains("low confidence"));
        },
        _ => panic!("Expected LowConfidence, got {:?}", result),
    }
}

#[test]
fn test_header_normalization_idempotency() {
    let input = "  Tên   công việc  ";
    let normalized = normalize_header(input);
    assert_eq!(normalized, "ten_cong_viec");
    
    // Idempotency check 
    let normalized_twice = normalize_header(&normalized);
    assert_eq!(normalized_twice, "ten_cong_viec");
}

#[test]
fn test_unit_normalization() {
    assert_eq!(normalize_unit("m2"), Some("m²".to_string()));
    assert_eq!(normalize_unit("mét vuông"), Some("m²".to_string()));
    assert_eq!(normalize_unit("unknown"), None);
}

#[test]
fn test_number_normalization() {
    assert_eq!(normalize_number(1.2345), 1.235);
    assert_eq!(normalize_number(1.2344), 1.234);
    assert_eq!(normalize_number(0.0001), 0.0); // 0.0001 * 1000 = 0.1 -> round 0
    assert_eq!(normalize_number(0.0005), 0.001); // 0.5 -> round 1 -> 0.001
}
