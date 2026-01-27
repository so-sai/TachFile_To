use serde::{Deserialize, Serialize};

/// ⚠️ CORE CONTRACT BOUNDARY
/// This file defines the strict data contract between TachFileTo (Ingestion)
/// and Iron Core (Business Logic).
///
/// DO NOT add business logic here. Data structs only.

#[derive(Debug, Serialize, Deserialize)]
pub struct IngestionObject {
    pub source: String,           // Always "tachfileto"
    pub project_uuid: String,     // UUID string (mock for now, real UUID in Phase 2)
    pub document_type: DocType,   // Resolution 254/2025
    pub content: String,          // Raw text extracted from Word/Excel
    pub checksum: String,         // SHA-256 integrity check
    pub origin_signature: String, // Ed25519 signature
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DocType {
    BoqRaw,
    ContractRaw,                 // Word ONSEN processing
    InvoiceMetadata,
    AcceptanceLogRaw,           // 2-day payment standard
}
