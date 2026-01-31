use iron_adapter::{Janitor, IronJanitor};
use iron_table::contract::{TableTruth, TableSchema, ColumnDef, DataType, TableRow, TableCell, CellValue, BoundingBox, ExtractionMeta};
use std::path::PathBuf;

#[test]
fn test_janitor_cleans_dirty_data() {
    // 1. Setup 'Dirty' TableTruth (as if from Docling)
    let schema = TableSchema {
        columns: vec![
            ColumnDef { name: "item".to_string(), dtype: DataType::Utf8, unit: None, nullable: false },
            ColumnDef { name: "quantity".to_string(), dtype: DataType::Float64, unit: Some("m³".to_string()), nullable: false },
            ColumnDef { name: "price".to_string(), dtype: DataType::Float64, unit: Some("VND".to_string()), nullable: false },
        ],
        row_count: 2,
        col_count: 3,
    };

    let dirty_row1 = TableRow {
        row_idx: 0,
        cells: vec![
            TableCell {
                row_idx: 0,
                col_idx: 0,
                value: CellValue::Text("Be tong mong".to_string()),
                bbox: BoundingBox { x: 0.0, y: 0.0, width: 10.0, height: 10.0, page: 1 },
                confidence: 0.9,
                source_text: "Be tong mong".to_string(),
            },
            TableCell {
                row_idx: 0,
                col_idx: 1,
                value: CellValue::Text("1.250,50 m3".to_string()), // Dirty: VN formatting + unit suffix
                bbox: BoundingBox { x: 0.0, y: 0.0, width: 10.0, height: 10.0, page: 1 },
                confidence: 0.85,
                source_text: "1.250,50 m3".to_string(),
            },
            TableCell {
                row_idx: 0,
                col_idx: 2,
                value: CellValue::Text("500.000 VND".to_string()), // Dirty: VN formatting + currency suffix
                bbox: BoundingBox { x: 0.0, y: 0.0, width: 10.0, height: 10.0, page: 1 },
                confidence: 0.88,
                source_text: "500.000 VND".to_string(),
            },
        ],
    };

    let dirty_row2 = TableRow {
        row_idx: 1,
        cells: vec![
            TableCell {
                row_idx: 1,
                col_idx: 0,
                value: CellValue::Text("Thep san".to_string()),
                bbox: BoundingBox { x: 0.0, y: 0.0, width: 10.0, height: 10.0, page: 1 },
                confidence: 0.9,
                source_text: "Thep san".to_string(),
            },
            TableCell {
                row_idx: 1,
                col_idx: 1,
                value: CellValue::Float(100.0),
                bbox: BoundingBox { x: 0.0, y: 0.0, width: 10.0, height: 10.0, page: 1 },
                confidence: 0.9,
                source_text: "100".to_string(),
            },
            TableCell {
                row_idx: 1,
                col_idx: 2,
                value: CellValue::Float(50.0),
                bbox: BoundingBox { x: 0.0, y: 0.0, width: 10.0, height: 10.0, page: 1 },
                confidence: 0.9,
                source_text: "50".to_string(),
            },
        ],
    };

    let dirty_table = TableTruth {
        table_id: "test_dirty".to_string(),
        source_file: PathBuf::from("test.pdf"),
        source_page: 1,
        schema,
        rows: vec![dirty_row1, dirty_row2],
        extraction_meta: ExtractionMeta {
            tool_version: "docling-mock".to_string(),
            timestamp: "2026-01-31".to_string(),
            confidence_score: 0.8,
        },
        bbox: BoundingBox { x: 0.0, y: 0.0, width: 100.0, height: 100.0, page: 1 },
    };

    // 2. Iron Table Truth should REJECT this dirty data if validated directly
    // (Because quantity and price are expected to be Float64 but contain Text strings)
    // Actually TableTruth doesn't check type consistency in validate_contract yet, it checks confidence.
    // But processing it in Engine would fail.

    // 3. Run Janitor
    let janitor = IronJanitor;
    let (clean_table, report) = janitor.cleanse(&dirty_table);

    // 4. Verify Cleansing
    println!("DEBUG: Janitor Report: {:#?}", report);
    for (i, row) in clean_table.rows.iter().enumerate() {
        println!("DEBUG: Row {}: {:?}", i, row.cells.iter().map(|c| &c.value).collect::<Vec<_>>());
    }

    assert_eq!(report.total_cells_cleaned, 2);
    
    let qty_cell = &clean_table.rows[0].cells[1];
    assert_eq!(qty_cell.value, CellValue::Float(1250.5));
    
    let price_cell = &clean_table.rows[0].cells[2];
    assert_eq!(price_cell.value, CellValue::Float(500000.0));

    // 5. Final Truth Check
    clean_table.validate_contract().expect("Contract should be valid after Janitor cleaning");
}
