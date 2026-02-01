use crate::core_contract::ui_contract::{FileStatus, FileStatusLabel, CellVerdict, VerdictLabel, DiscrepancySummary, EvidenceData};
use crate::ForensicState;
use iron_table::contract::{RejectionReason, LineageEntry};
use iron_adapter::diagnostics::{DiagnosticEngine, StructuredRejection};
use tauri::State;
use tracing::instrument;

#[tauri::command]
#[instrument(skip_all)]
pub fn get_file_ledger() -> Vec<FileStatus> {
    vec![
        FileStatus {
            name: "Tri-Conflict-Pack.pdf".to_string(),
            status: FileStatusLabel::Tainted,
            timestamp: "2026-01-31 22:50".to_string(),
        },
        FileStatus {
            name: "BM_01_Kiem_Dinh.pdf".to_string(),
            status: FileStatusLabel::Clean,
            timestamp: "2026-01-31 20:00".to_string(),
        },
        FileStatus {
            name: "BM_03_Thanh_Toan.pdf".to_string(),
            status: FileStatusLabel::Rejected,
            timestamp: "2026-01-31 20:30".to_string(),
        },
    ]
}

#[tauri::command]
#[instrument(skip(state), fields(file_id = %fileId))]
pub fn get_table_truth(
    #[allow(non_snake_case)] fileId: String, 
    state: State<'_, ForensicState>
) -> Vec<CellVerdict> {
    let file_id = fileId; // Renamed for Rust convention internally
    // 1. Ingest/Swap table if necessary
    {
        let Ok(mut table_guard) = state.active_table.lock() else {
            tracing::error!("Failed to acquire table lock");
            return vec![];
        };
        let should_swap = table_guard.as_ref()
            .map(|t| t.table_id != file_id)
            .unwrap_or(true);
        if should_swap {
            if file_id == "Tri-Conflict-Pack.pdf" {
                *table_guard = Some(generate_tri_conflict_table());
            } else {
                *table_guard = Some(generate_default_table(file_id));
            }
        }
    }

    let Ok(table_guard) = state.active_table.lock() else {
        tracing::error!("Failed to acquire table lock for reading");
        return vec![];
    };
    let table = match table_guard.as_ref() {
        Some(t) => t,
        None => return vec![],
    };

    // Run real diagnostics
    let violations = DiagnosticEngine::diagnose(table);
    
    // Map TableTruth + Violations to CellVerdicts
    let mut grid = Vec::new();
    
    // Store violations in state for subsequent use
    let Ok(mut violations_guard) = state.active_violations.lock() else {
        tracing::error!("Failed to acquire violations lock");
        return vec![];
    };
    *violations_guard = violations.clone();

    for row in &table.rows {
        for cell in &row.cells {
            let cell_id = format!("cell_{}_{}", cell.row_idx, cell.col_idx);
            
            // Find if this cell has a violation
            let violation = violations.iter().find(|v| match v {
                StructuredRejection::EncodingCorruption { row, col, .. } => *row == cell.row_idx && *col == cell.col_idx,
                StructuredRejection::LowConfidence { row, col, .. } => *row == cell.row_idx && *col == cell.col_idx,
                StructuredRejection::TypeMismatch { row, col, .. } => *row == cell.row_idx && *col == cell.col_idx,
                _ => false,
            });

            let (verdict, reason) = if let Some(v) = violation {
                match v {
                    StructuredRejection::EncodingCorruption { category, .. } => (VerdictLabel::Inadmissible, Some(*category)),
                    StructuredRejection::LowConfidence { category, .. } => (VerdictLabel::Inadmissible, Some(*category)),
                    StructuredRejection::TypeMismatch { category, .. } => (VerdictLabel::Inadmissible, Some(*category)),
                    _ => (VerdictLabel::Admissible, None),
                }
            } else {
                (VerdictLabel::Admissible, None)
            };

            grid.push(CellVerdict {
                cell_id,
                value: Some(format!("{:?}", cell.value)), // Simplistic formatting
                verdict,
                reason,
                row_idx: cell.row_idx,
                col_idx: cell.col_idx,
                source_text: cell.source_text.clone(),
            });
        }
    }

    grid
}

#[tauri::command]
#[instrument(skip(state))]
pub fn get_discrepancy(state: State<'_, ForensicState>) -> DiscrepancySummary {
    let Ok(violations) = state.active_violations.lock() else {
        tracing::error!("Failed to acquire violations lock for discrepancy");
        return DiscrepancySummary { consistent: 0, inconsistent: 0, requires_review: 0 };
    };
    let Ok(table_guard) = state.active_table.lock() else {
        tracing::error!("Failed to acquire table lock for discrepancy");
        return DiscrepancySummary { consistent: 0, inconsistent: 0, requires_review: 0 };
    };
    
    let total_cells = table_guard.as_ref().map(|t| t.schema.row_count * t.schema.col_count).unwrap_or(0);
    let inconsistent = violations.len();
    let requires_review = violations.iter().filter(|v| matches!(v, StructuredRejection::LowConfidence { .. })).count();

    DiscrepancySummary {
        consistent: total_cells.saturating_sub(inconsistent),
        inconsistent,
        requires_review,
    }
}

#[tauri::command]
#[instrument(skip(state), fields(cell_id = %cellId))]
pub fn get_evidence(
    #[allow(non_snake_case)] cellId: String,
    state: State<'_, ForensicState>
) -> EvidenceData {
    let cell_id = cellId; // Renamed for Rust convention internally
    let Ok(table_guard) = state.active_table.lock() else {
        tracing::error!("Failed to acquire table lock for evidence");
        return EvidenceData { image_base64: "".to_string(), metadata: "Lock error".to_string() };
    };
    let table = match table_guard.as_ref() {
        Some(t) => t,
        None => return EvidenceData { image_base64: "".to_string(), metadata: "No table loaded".to_string() },
    };

    let parts: Vec<&str> = cell_id.split('_').collect();
    if parts.len() < 3 {
        return EvidenceData { image_base64: "".to_string(), metadata: "Invalid cell_id".to_string() };
    }
    let row_idx: usize = parts[1].parse().unwrap_or(0);
    let col_idx: usize = parts[2].parse().unwrap_or(0);

    if let Some(row) = table.rows.get(row_idx) {
        if let Some(cell) = row.cells.get(col_idx) {
            let bbox = &cell.bbox;
            let page_idx = bbox.page;

            // In Mission 026 Stress Test, if the file doesn't exist, we return a mock visual pattern
            // pointing out the discrepancy if it's the Tri-Conflict-Pack.
            if table.table_id == "Tri-Conflict-Pack.pdf" && !table.source_file.exists() {
                return EvidenceData {
                    image_base64: "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==".to_string(),
                    metadata: format!("STRESS_TEST_MODE: Proof for {},{} | Page {}", row_idx, col_idx, page_idx + 1),
                };
            }

            match elite_pdf::EliteDocument::new(table.source_file.to_string_lossy().to_string()) {
                Ok(doc) => {
                    match elite_pdf::ElitePage::from_document(&doc, page_idx as i32) {
                        Ok(page) => {
                            let base64_image = page.get_crop_base64(bbox.x, bbox.y, bbox.x + bbox.width, bbox.y + bbox.height, 2.0)
                                .unwrap_or_else(|_| "".to_string());

                            return EvidenceData {
                                image_base64: base64_image,
                                metadata: format!("Evidence for cell {},{} | Page {}", row_idx, col_idx, page_idx + 1),
                            };
                        }
                        Err(e) => return EvidenceData { image_base64: "".to_string(), metadata: format!("Page load error: {}", e) },
                    }
                }
                Err(e) => return EvidenceData { image_base64: "".to_string(), metadata: format!("Doc open error: {}", e) },
            }
        }
    }

    EvidenceData { image_base64: "".to_string(), metadata: "Cell not found".to_string() }
}

#[tauri::command]
#[instrument(skip(state))]
pub fn get_metric_lineage(
    metric_key: String,
    state: State<'_, ForensicState>
) -> Vec<LineageEntry> {
    let Ok(truth_guard) = state.active_project_truth.lock() else {
        return vec![];
    };
    
    if let Some(truth) = truth_guard.as_ref() {
        truth.lineage.get(&metric_key).cloned().unwrap_or_default()
    } else {
        vec![]
    }
}

// --- MISSION 026: STRESS TEST GENERATORS ---

fn generate_tri_conflict_table() -> iron_table::contract::TableTruth {
    use iron_table::contract::*;
    use std::path::PathBuf;

    let columns = vec![
        ColumnDef { name: "STT".to_string(), dtype: DataType::Int, unit: None, nullable: false, is_critical: false },
        ColumnDef { name: "Description".to_string(), dtype: DataType::Utf8, unit: None, nullable: false, is_critical: true },
        ColumnDef { name: "Amount".to_string(), dtype: DataType::Float64, unit: Some("VND".to_string()), nullable: false, is_critical: true },
    ];

    let rows = vec![
        TableRow { row_idx: 0, cells: vec![
            TableCell { global_id: "Tri-Conflict_0_0".to_string(), row_idx: 0, col_idx: 0, value: CellValue::Int(1), bbox: BoundingBox { x: 50.0, y: 100.0, width: 20.0, height: 15.0, page: 0 }, confidence: 1.0, source_text: "1".to_string(), encoding_status: EncodingStatus::Clean, encoding_evidence: None },
            TableCell { global_id: "Tri-Conflict_0_1".to_string(), row_idx: 0, col_idx: 1, value: CellValue::Text("Hạng mục A".to_string()), bbox: BoundingBox { x: 75.0, y: 100.0, width: 200.0, height: 15.0, page: 0 }, confidence: 1.0, source_text: "Hạng mục A".to_string(), encoding_status: EncodingStatus::Clean, encoding_evidence: None },
            TableCell { global_id: "Tri-Conflict_0_2".to_string(), row_idx: 0, col_idx: 2, value: CellValue::Float(1000.0), bbox: BoundingBox { x: 280.0, y: 100.0, width: 100.0, height: 15.0, page: 0 }, confidence: 1.0, source_text: "1,000".to_string(), encoding_status: EncodingStatus::Clean, encoding_evidence: None },
        ]},
        TableRow { row_idx: 1, cells: vec![
            TableCell { global_id: "Tri-Conflict_1_0".to_string(), row_idx: 1, col_idx: 0, value: CellValue::Int(2), bbox: BoundingBox { x: 50.0, y: 120.0, width: 20.0, height: 15.0, page: 0 }, confidence: 1.0, source_text: "2".to_string(), encoding_status: EncodingStatus::Clean, encoding_evidence: None },
            TableCell { global_id: "Tri-Conflict_1_1".to_string(), row_idx: 1, col_idx: 1, value: CellValue::Text("Hạng mục B".to_string()), bbox: BoundingBox { x: 75.0, y: 120.0, width: 200.0, height: 15.0, page: 0 }, confidence: 1.0, source_text: "Hạng mục B".to_string(), encoding_status: EncodingStatus::Clean, encoding_evidence: None },
            TableCell { global_id: "Tri-Conflict_1_2".to_string(), row_idx: 1, col_idx: 2, value: CellValue::Float(2000.0), bbox: BoundingBox { x: 280.0, y: 120.0, width: 100.0, height: 15.0, page: 0 }, confidence: 1.0, source_text: "2,000".to_string(), encoding_status: EncodingStatus::Clean, encoding_evidence: None },
        ]},
        TableRow { row_idx: 2, cells: vec![
            TableCell { global_id: "Tri-Conflict_2_0".to_string(), row_idx: 2, col_idx: 0, value: CellValue::Null, bbox: BoundingBox { x: 50.0, y: 140.0, width: 20.0, height: 15.0, page: 0 }, confidence: 1.0, source_text: "".to_string(), encoding_status: EncodingStatus::Clean, encoding_evidence: None },
            TableCell { global_id: "Tri-Conflict_2_1".to_string(), row_idx: 2, col_idx: 1, value: CellValue::Text("Tổng cộng (Summary Page)".to_string()), bbox: BoundingBox { x: 75.0, y: 140.0, width: 200.0, height: 15.0, page: 1 }, confidence: 1.0, source_text: "Tổng cộng".to_string(), encoding_status: EncodingStatus::Clean, encoding_evidence: None },
            TableCell { global_id: "Tri-Conflict_2_2".to_string(), row_idx: 2, col_idx: 2, value: CellValue::Float(2900.0), bbox: BoundingBox { x: 280.0, y: 140.0, width: 100.0, height: 15.0, page: 1 }, confidence: 1.0, source_text: "2,900".to_string(), encoding_status: EncodingStatus::Clean, encoding_evidence: None },
        ]},
    ];

    TableTruth {
        table_id: "Tri-Conflict-Pack.pdf".to_string(),
        source_file: PathBuf::from("Tri-Conflict-Pack.pdf"),
        source_page: 0,
        schema: TableSchema { columns, row_count: 3, col_count: 3 },
        rows,
        extraction_meta: ExtractionMeta { tool_version: "2.5".to_string(), timestamp: "2026-01-31".to_string(), confidence_score: 1.0 },
        bbox: BoundingBox { x: 0.0, y: 0.0, width: 595.0, height: 842.0, page: 0 },
    }
}

fn generate_default_table(file_id: String) -> iron_table::contract::TableTruth {
    use iron_table::contract::*;
    use std::path::PathBuf;

    TableTruth {
        table_id: file_id.clone(),
        source_file: PathBuf::from(file_id),
        source_page: 0,
        schema: TableSchema { columns: vec![], row_count: 0, col_count: 0 },
        rows: vec![],
        extraction_meta: ExtractionMeta { tool_version: "2.5".to_string(), timestamp: "2026-01-31".to_string(), confidence_score: 1.0 },
        bbox: BoundingBox { x: 0.0, y: 0.0, width: 100.0, height: 100.0, page: 0 },
    }
}
