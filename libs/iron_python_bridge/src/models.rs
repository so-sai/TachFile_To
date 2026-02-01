use serde::{Deserialize, Serialize};

/// Metadata for an individual page, mapping 1-1 with Python `PageMetadata`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PageMetadata {
    pub page: usize,
    pub content_type: String,
    pub confidence: f64,
    pub text_length: usize,
    pub table_rows: Option<usize>,
    pub table_columns: Option<usize>,
}

/// The "Canonical Truth" for Mission 014 extraction.
/// All lanes (PDF, Office, MD) must return this unified struct.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractionProduct {
    pub source: String,
    pub checksum: String,
    pub lane: String,          // e.g. "PDF_OCR", "XLSX_POLARS"
    pub content: String,       // Markdown (Mininal Truth)
    pub evidence: serde_json::Value, // Metadata (Coordinates, Raw JSON)
    pub pages: Vec<PageMetadata>,
    pub performance_metrics: ExtractionMetrics,
    pub schema_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractionMetrics {
    pub total_ms: i64,
    pub lane_ms: i64,          // Time spent in the specific worker
    pub worker_restarts: u32,  // Resilience tracking
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_extraction_product() {
        let json_data = r##"{
            "source": "test_path/contract.pdf",
            "checksum": "sha256:abc123def456",
            "lane": "PDF_OCR",
            "content": "# Markdown Header",
            "evidence": {"raw": "data"},
            "pages": [
                {
                    "page": 1,
                    "content_type": "table",
                    "confidence": 0.98,
                    "text_length": 150
                }
            ],
            "performance_metrics": {
                "total_ms": 1000,
                "lane_ms": 800,
                "worker_restarts": 0
            },
            "schema_version": "MF50-EP-0.1"
        }"##;

        let result: ExtractionProduct = serde_json::from_str(json_data).expect("Should deserialize correctly");
        
        assert_eq!(result.lane, "PDF_OCR");
        assert_eq!(result.performance_metrics.lane_ms, 800);
        assert!(result.content.contains("Markdown Header"));
    }
}
