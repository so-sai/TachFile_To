use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata for an individual page, mapping 1-1 with Python `PageMetadata`
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PageMetadata {
    pub page: usize,
    pub content_type: String,
    pub confidence: f64,
    pub text_length: usize,
    pub table_rows: Option<usize>,
    pub table_columns: Option<usize>,
}

/// Complete ingestion result, mapping 1-1 with Python `IngestionResult`
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct IngestionResult {
    pub source: String,
    pub checksum: String,
    pub pages: Vec<PageMetadata>,
    pub raw_content: Option<String>,
    pub tables: Vec<serde_json::Value>,
    pub extraction_meta: HashMap<String, serde_json::Value>,
    pub schema_version: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_ingestion_result() {
        let json_data = r#"{
            "source": "test_path/contract.pdf",
            "checksum": "sha256:abc123def456",
            "pages": [
                {
                    "page": 1,
                    "content_type": "table",
                    "confidence": 0.98,
                    "text_length": 150
                },
                {
                    "page": 2,
                    "content_type": "text",
                    "confidence": 0.95,
                    "text_length": 500
                }
            ],
            "extraction_meta": {
                "engine": "docling",
                "engine_version": "2.68.0",
                "format": "pdf"
            },
            "raw_content": "header content",
            "tables": [],
            "schema_version": "MF50-INGEST-0.1"
        }"#;

        let result: IngestionResult = serde_json::from_str(json_data).expect("Should deserialize correctly");
        
        assert_eq!(result.source, "test_path/contract.pdf");
        assert_eq!(result.checksum, "sha256:abc123def456");
        assert_eq!(result.pages.len(), 2);
        assert_eq!(result.pages[0].content_type, "table");
        assert_eq!(result.pages[0].page, 1);
        assert_eq!(result.extraction_meta.get("engine").unwrap(), "docling");
    }
}
