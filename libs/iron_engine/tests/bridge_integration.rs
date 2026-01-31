use iron_engine::{to_dataframe, TableTruth};
use iron_adapter::Janitor;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_bridge_separation_of_powers() {
    // 1. Load the "Docling V2 Construction Noise" mock
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/mocks/docling_v2_export.json");
    
    let json_content = fs::read_to_string(&path)
        .expect("Should be able to read mock file");
        
    let mut table: TableTruth = serde_json::from_str(&json_content)
        .expect("Mock JSON should be deserializable as TableTruth");

    // 2. PROVE STRICT REJECTION: Truth layer should REJECT dirty data without Janitor
    let initial_validation = table.validate_contract();
    assert!(initial_validation.is_err(), "Truth layer should have rejected dirty data by default");
    println!("SUCCESS: Truth layer correctly rejected dirty mock before cleaning.");

    // 3. APPLY JANITOR (Adapter Layer)
    // Janitor cleans the chaos so the Truth Layer can accept it.
    Janitor::clean(&mut table);

    // 4. VALIDATE TRUTH (The Gate)
    // After Janitor does its job, the Truth layer SHOULD pass.
    if let Err(e) = table.validate_contract() {
        panic!("Contract validation failed AFTER Janitor cleaning: {:?}", e);
    }
    println!("SUCCESS: Truth layer accepted data AFTER Janitor cleaning.");
    
    // 4. Attempt Iron Engine Transformation
    let df_result = to_dataframe(&table);
    
    match df_result {
        Ok(df) => {
            // Check Unit Stripping Recovery
            let qty_series = df.column("QUANTITY").unwrap().f64().unwrap();
            
            // Row 0: 150.5 (was clean)
            assert_eq!(qty_series.get(0), Some(150.5));
            
            // Row 1: "1.250,50 m3" -> Should be 1250.5
            let val_r1 = qty_series.get(1);
            if let Some(v) = val_r1 {
                assert!((v - 1250.5).abs() < 0.001, "Unit extraction failed. Expected 1250.5, got {}", v);
                println!("SUCCESS: Recovered 1250.5 from '1.250,50 m3'");
            } else {
                panic!("Unit extraction failed: Got Null instead of 1250.5");
            }
            
            // Row 2: "80" (was clean) -> 80.0
             assert_eq!(qty_series.get(2), Some(80.0));
            
            // Check Ghost Column Silencing
            let ghost_series = df.column("GHOST_COL").unwrap().str().unwrap();
            
            // The Ghost Col was low confidence (< 0.7).
            // Janitor should have converted them to Null, PRESERVING the low confidence.
            // validate_contract in iron_table must be aware that if a value is Null, 
            // the confidence check might behave differently or we might need to adjust it.
            // For now, let's verify it hits DataFrame as Null.
            assert_eq!(ghost_series.null_count(), ghost_series.len(), "Ghost column should be silenced to all nulls");
            
            println!("SUCCESS: Sanitization Gate passed all checks.");
        },
        Err(e) => {
            panic!("Engine failed to process sanitized table: {:?}", e);
        }
    }
}
