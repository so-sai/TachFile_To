//! Integration tests for Repair Loop (Mission 025: The Repair Gauntlet)

use iron_adapter::{TableTruth, CellValue, RepairEngine, CellRepair, DiagnosticEngine};
use iron_adapter::docling_bridge::DoclingBridge;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/fixtures");
    d.push(name);
    d
}

#[test]
fn test_025_01_user_fixes_low_confidence() {
    // SCENARIO: Cell has 0.5 confidence. Law rejects.
    // ACTION: User proofreads and submits a Repair.
    // EXPECTATION: Repaired cell has 1.0 confidence and Law ACCEPTS.

    let path = fixture("docling_dirty.json"); 
    let tables = DoclingBridge::ingest(&path).expect("Ingest should succeed");
    let target_table = &tables[0];

    // Verify initial rejection
    assert!(target_table.validate_contract().is_err(), "Initial data should be rejected");

    // Create repairs for BOTH low confidence cells (0.5 < 0.7)
    let repair_1 = CellRepair {
        row_idx: 0,
        col_idx: 0,
        old_value: CellValue::Text("Co lumn 1".to_string()),
        new_value: CellValue::Text("Column 1".to_string()),
        reason: "Fixed OCR typo".to_string(),
    };
    let repair_2 = CellRepair {
        row_idx: 1,
        col_idx: 0,
        old_value: CellValue::Text("Data Row".to_string()),
        new_value: CellValue::Text("Data Row".to_string()),
        reason: "Confirmed correct".to_string(),
    };

    // Apply repairs
    let result = RepairEngine::apply_repairs(target_table, vec![repair_1, repair_2], None);
    assert!(result.is_ok(), "Repairs should be applied successfully: {:?}", result.err());
    
    let repaired_table = result.unwrap();
    
    // LAW CHECK: Re-run validation
    assert!(repaired_table.validate_contract().is_ok(), "Repaired table should pass validation");
    
    // Verification: Confidence should be 1.0
    let cell = &repaired_table.rows[0].cells[0];
    assert_eq!(cell.confidence, 1.0);
    assert_eq!(cell.value, CellValue::Text("Column 1".to_string()));
}

#[test]
fn test_025_02_user_fixes_sum_mismatch() {
    // SCENARIO: Summary (100) vs Detail (90). Project REJECTS.
    // ACTION: User fixes detail item (40 -> 50).
    // EXPECTATION: Project validation ACCEPTS.

    use iron_table::project::{ProjectTruth, ConsistencyRule, TableRef, ColumnRef, CellRef};

    let mut project = ProjectTruth::new("Mission025".to_string());
    
    // Ingest summary (Total = 100) and detail (Sum = 90)
    let sum_path = fixture("clean_summary.json");
    let det_path = fixture("clean_detail_mismatch.json");
    
    let tables_sum = DoclingBridge::ingest(&sum_path).unwrap();
    let tables_det = DoclingBridge::ingest(&det_path).unwrap();
    
    project.add_table(tables_sum[0].clone());
    project.add_table(tables_det[0].clone());
    
    // Add SumMatch Rule: Sum(Detail[Col 1]) == Summary[Row 1, Col 1]
    // clean_summary.json Row 1, Col 1 is 100.0 (based on previous mission context)
    project.add_rule(ConsistencyRule::SumMatch {
        source_table: TableRef(tables_det[0].table_id.clone()),
        source_column: ColumnRef { table_id: tables_det[0].table_id.clone(), col_idx: 1 },
        target: CellRef { table_id: tables_sum[0].table_id.clone(), row_idx: 1, col_idx: 1 },
    });

    // Verify initial project rejection
    let validation = project.validate_project();
    println!("DEBUG: Initial validation result: {:?}", validation);
    assert!(validation.is_err(), "Project should reject 90 != 100");

    // Human Repair: Change the detail row that was 40 to 50
    // clean_detail_mismatch.json: 
    // Row 1: Item A, 50
    // Row 2: Item B, 40
    let repair = CellRepair {
        row_idx: 2, // Item B
        col_idx: 1,
        old_value: CellValue::Float(40.0),
        new_value: CellValue::Float(50.0),
        reason: "User corrected missing 10 units in detail".to_string(),
    };

    let repaired_det = RepairEngine::apply_repairs(&tables_det[0], vec![repair], None).expect("Repair Detail failed");
    
    // Update project with repaired table
    project.add_table(repaired_det);

    // LAW CHECK: Re-run global project validation
    assert!(project.validate_project().is_ok(), "Project should accept after repair (100 == 100)");
}

#[test]
fn test_025_03_bypass_attempt_rejected() {
    // SCENARIO: User tries to fix a cell but provides invalid encoding (Mojibake).
    // EXPECTATION: RepairEngine reflects the gatekeeper scanning during re-validation.

    let path = fixture("clean_summary.json");
    let table = &DoclingBridge::ingest(&path).unwrap()[0];

    // clean_summary Row 1, Col 1 is "100" (parsed as Int)
    let dirty_repair = CellRepair {
        row_idx: 1,
        col_idx: 1,
        old_value: CellValue::Int(100),
        new_value: CellValue::Text("\u{251C}\u{00AC}".to_string()), // Forced Mojibake: ├¬
        reason: "Intentionally broken repair".to_string(),
    };

    let result = RepairEngine::apply_repairs(table, vec![dirty_repair], None);
    
    match result {
        Err(iron_adapter::TableRejection::EncodingCorruption(_)) => {
            // Success! The gatekeeper caught the human injection
        }
        Err(e) => panic!("Expected EncodingCorruption, got: {:?}", e),
        Ok(_) => panic!("Mojibake repair should have been rejected by re-validation gate!"),
    }
}
