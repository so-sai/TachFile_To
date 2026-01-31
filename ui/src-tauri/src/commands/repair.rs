use iron_adapter::diagnostics::{StructuredRejection, DiagnosticEngine, CellRepair, TruthSnapshot};
use iron_adapter::repair_engine::RepairEngine;
use iron_table::contract::{TableTruth, TableRejection};

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
