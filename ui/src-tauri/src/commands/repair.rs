use iron_adapter::diagnostics::{StructuredRejection, DiagnosticEngine, CellRepair, TruthSnapshot};
use iron_adapter::repair_engine::{RepairEngine, EncodingCandidate};
use iron_table::contract::{TableTruth, TableRejection};
use crate::ForensicState;
use tauri::State;

#[tauri::command]
pub async fn apply_table_repairs_to_active(
    repair: CellRepair,
    state: State<'_, ForensicState>
) -> Result<(), String> {
    let mut table_guard = state.active_table.lock().map_err(|e| e.to_string())?;
    let table = table_guard.as_mut().ok_or_else(|| "No active table".to_string())?;
    
    let repaired_table = RepairEngine::apply_repairs(table, vec![repair], None)
        .map_err(|e| e.to_string())?;
    
    *table = repaired_table;
    Ok(())
}

#[tauri::command]
pub async fn get_encoding_candidates(text: String) -> Vec<EncodingCandidate> {
    RepairEngine::get_encoding_candidates(&text)
}

#[tauri::command]
pub async fn get_structural_diagnostics(table: TableTruth) -> Vec<StructuredRejection> {
    DiagnosticEngine::diagnose(&table)
}

#[tauri::command]
pub async fn apply_table_repairs(
    table: TableTruth, 
    repairs: Vec<CellRepair>
) -> Result<TableTruth, String> {
    RepairEngine::apply_repairs(&table, repairs, None)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn seal_table_truth(
    table_id: String,
    raw_input: String,
    repairs: Vec<CellRepair>,
    virtual_truth: TableTruth,
    actor: String,
    verdict: String
) -> TruthSnapshot {
    RepairEngine::seal_truth(&table_id, &raw_input, &repairs, &virtual_truth, &actor, &verdict)
}
