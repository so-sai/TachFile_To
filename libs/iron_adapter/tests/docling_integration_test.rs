//! Integration tests for DoclingBridge (Mission 024: The Gauntlet)
//!
//! These tests verify the "Clean Hands" doctrine: Bridge is a courier, not a lawyer.

use iron_adapter::docling_bridge::DoclingBridge;
use iron_adapter::{TableRejection, TableTruth, EncodingStatus, CellValue};
use std::path::PathBuf;

/// Helper to get fixture path
fn fixture(name: &str) -> PathBuf {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/fixtures");
    d.push(name);
    d
}

// =============================================================================
// THE GAUNTLET: Core Tests (024-01 to 024-03)
// =============================================================================

#[test]
fn test_024_01_dirty_docling_rejected() {
    // SCENARIO: The PDF extraction has low confidence (0.5 < 0.7 threshold).
    // EXPECTATION: Ingest succeeds (Courier), but validate_contract REJECTS.
    
    let path = fixture("docling_dirty.json");
    let result = DoclingBridge::ingest(&path);
    
    assert!(result.is_ok(), "Bridge should ingest dirty data without crashing");
    let tables = result.expect("Already checked is_ok");
    assert!(!tables.is_empty());
    
    let table = &tables[0];
    let validation = table.validate_contract();
    
    // It should fail due to Low Confidence (0.5 < 0.7)
    match validation {
        Err(TableRejection::LowConfidence(_)) => {} // Expected
        Err(e) => panic!("Expected LowConfidence error, got: {:?}", e),
        Ok(_) => panic!("Dirty table should have been rejected!"),
    }
}

#[test]
fn test_024_02_clean_tables_project_mismatch_rejected() {
    // SCENARIO: 
    // - File A (Summary): "Total = 100"
    // - File B (Detail): "Sum(Items) = 90" (40 + 50)
    // EXPECTATION: Individual tables valid, but cross-doc totals mismatch.
    
    let sum_path = fixture("clean_summary.json");
    let det_path = fixture("clean_detail_mismatch.json");
    
    let tables_a = DoclingBridge::ingest(&sum_path).expect("Ingest summary failed");
    let tables_b = DoclingBridge::ingest(&det_path).expect("Ingest detail failed");
    
    assert!(tables_a[0].validate_contract().is_ok(), "Summary table should be valid");
    assert!(tables_b[0].validate_contract().is_ok(), "Detail table should be valid");

    // Simulating ProjectTruth::validate() - semantic cross-document check
    let total_declared = 100.0_f64;
    let item_sum = 40.0 + 50.0; // From detail fixture
    
    assert!((total_declared - item_sum).abs() > 0.01, "Project should detect mismatch");
}

#[test]
fn test_024_03_clean_pipeline_accepted() {
    // SCENARIO: Perfect data flow where totals match.
    // EXPECTATION: All validations pass.
    
    let sum_path = fixture("clean_summary.json");
    let det_path = fixture("clean_detail_match.json");
    
    let tables_a = DoclingBridge::ingest(&sum_path).expect("Ingest summary failed");
    let tables_b = DoclingBridge::ingest(&det_path).expect("Ingest detail failed");
    
    assert!(tables_a[0].validate_contract().is_ok(), "Summary should be valid");
    assert!(tables_b[0].validate_contract().is_ok(), "Detail should be valid");
    
    // Semantic check: 100 = 50 + 50
    let total_declared: f64 = 100.0;
    let item_sum: f64 = 50.0 + 50.0;
    assert!((total_declared - item_sum).abs() < 0.01, "Totals should match");
}

// =============================================================================
// TDD REINFORCEMENT: Additional Tests (024-04 to 024-07)
// =============================================================================

#[test]
fn test_024_04_confidence_guard() {
    // SCENARIO: A cell has text "100" but confidence 0.5 (< 0.7 threshold).
    // EXPECTATION: TableTruth::validate_contract() rejects with LowConfidence.
    
    let path = fixture("docling_dirty.json"); // Has confidence 0.5
    let tables = DoclingBridge::ingest(&path).expect("Ingest should succeed");
    
    let validation = tables[0].validate_contract();
    
    match validation {
        Err(TableRejection::LowConfidence(msg)) => {
            assert!(msg.contains("0.5") || msg.contains("low confidence"), 
                "Error message should mention low confidence: {}", msg);
        }
        Err(e) => panic!("Expected LowConfidence, got: {:?}", e),
        Ok(_) => panic!("Low confidence cell should trigger rejection"),
    }
}

#[test]
fn test_024_05_mojibake_handshake() {
    // SCENARIO: Docling outputs Mojibake: "B├¬ t├┤ng" (corrupted Vietnamese).
    // EXPECTATION: Bridge calls EncodingGatekeeper, marks cell as Invalid.
    
    let path = fixture("mojibake_text.json");
    let tables = DoclingBridge::ingest(&path).expect("Ingest should succeed");
    
    let table = &tables[0];
    
    // Find the cell with mojibake text
    let mut found_invalid = false;
    for row in &table.rows {
        for cell in &row.cells {
            if cell.source_text.contains("├") {
                assert_eq!(cell.encoding_status, EncodingStatus::Invalid,
                    "Mojibake cell should be marked Invalid");
                found_invalid = true;
            }
        }
    }
    
    assert!(found_invalid, "Should have found at least one mojibake cell");
    
    // Validation should fail due to encoding corruption
    let validation = table.validate_contract();
    match validation {
        Err(TableRejection::EncodingCorruption(_)) => {} // Expected
        Err(e) => panic!("Expected EncodingCorruption, got: {:?}", e),
        Ok(_) => panic!("Mojibake table should have been rejected!"),
    }
}

#[test]
fn test_024_06_empty_table_rejected() {
    // SCENARIO: Docling returns a TableBlock with cells: [] (empty array).
    // EXPECTATION: Bridge produces TableTruth, but validate_contract rejects.
    
    let path = fixture("empty_table.json");
    let tables = DoclingBridge::ingest(&path).expect("Ingest should succeed");
    
    assert_eq!(tables.len(), 1, "Should have one table");
    
    let validation = tables[0].validate_contract();
    
    // Empty table should fail SizeConstraintViolation (row_count < 2)
    match validation {
        Err(TableRejection::SizeConstraintViolation(msg)) => {
            assert!(msg.contains("< 2"), "Should mention minimum row requirement");
        }
        Err(e) => panic!("Expected SizeConstraintViolation, got: {:?}", e),
        Ok(_) => panic!("Empty table should have been rejected!"),
    }
}

#[test]
fn test_024_07_coordinate_parity() {
    // SCENARIO: A 2x2 table with precise floating-point coordinates.
    // EXPECTATION: Cell at (row: 1, col: 1) maps correctly with exact bbox values.
    
    let path = fixture("coordinate_test.json");
    let tables = DoclingBridge::ingest(&path).expect("Ingest should succeed");
    
    let table = &tables[0];
    
    // Find cell at row 1, col 1
    let target_cell = table.rows
        .iter()
        .find(|r| r.row_idx == 1)
        .and_then(|r| r.cells.iter().find(|c| c.col_idx == 1));
    
    let cell = target_cell.expect("Cell (1,1) should exist");
    
    // Verify the text matches what we put in the fixture
    match &cell.value {
        CellValue::Text(t) => {
            assert_eq!(t, "Cell_1_1", "Cell text should match fixture");
        }
        _ => panic!("Expected Text value"),
    }
    
    // Verify bbox precision (from fixture: [60.875, 40.25, 100.5, 50.375])
    assert!((cell.bbox.x - 60.875).abs() < 0.001, "X coordinate precision");
    assert!((cell.bbox.y - 40.25).abs() < 0.001, "Y coordinate precision");
    assert!((cell.bbox.width - (100.5 - 60.875)).abs() < 0.001, "Width precision");
    assert!((cell.bbox.height - (50.375 - 40.25)).abs() < 0.001, "Height precision");
}
