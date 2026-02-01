use serde::{Deserialize, Serialize};
use iron_table::contract::RejectionReason;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UITruthContract {
    pub file_ledger: Vec<FileStatus>,
    pub table_truth: Vec<CellVerdict>,
    // pub evidence: EvidenceMap, // TODO: Implement EvidenceMap
    pub discrepancy: DiscrepancySummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileStatus {
    pub name: String,
    pub status: FileStatusLabel,
    pub timestamp: String,
    pub progress: Option<f64>, // 0.0 to 100.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FileStatusLabel {
    Clean,
    Tainted,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellVerdict {
    #[serde(rename = "cell_id")]
    pub cell_id: String,
    pub value: Option<String>,
    pub verdict: VerdictLabel,
    pub reason: Option<RejectionReason>,
    #[serde(rename = "row_idx")]
    pub row_idx: usize,
    #[serde(rename = "col_idx")]
    pub col_idx: usize,
    #[serde(rename = "source_text")]
    pub source_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VerdictLabel {
    Admissible,
    Inadmissible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscrepancySummary {
    pub consistent: usize,
    pub inconsistent: usize,
    pub requires_review: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceData {
    pub image_base64: String,
    pub metadata: String, // Contextual metadata
}
