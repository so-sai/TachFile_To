use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageType {
    CmdHandshake,
    ResHandshake,
    CmdPing,
    ResPong,
    CmdExtractEvidence,
    CmdParseTable,
    ResSuccess,
    ResProgress,
    ResError,
    CmdShutdown,
    ResAck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CacheHit {
    Memory,
    Disk,
    Miss,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage<T> {
    pub protocol_v: String,
    pub msg_id: Uuid,
    pub timestamp: i64,
    #[serde(rename = "type")]
    pub msg_type: MessageType,
    pub payload: T,
}

// Payload for CMD_HANDSHAKE
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeRequestPayload {
    pub rust_version: String,
    pub expected_protocol_v: String,
    pub capabilities_requested: Vec<String>,
}

// Payload for RES_HANDSHAKE
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeResponsePayload {
    pub worker_pid: u32,
    pub docling_version: String,
    pub python_version: String,
    pub capabilities_supported: Vec<String>,
    pub max_memory_mb: u32,
    pub status: String,
}

// Payload for CMD_EXTRACT_EVIDENCE
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractEvidencePayload {
    pub file_path: PathBuf,
    pub page_index: usize,
    pub bbox: BoundingBox,
    pub dpi: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    #[serde(default = "default_unit")]
    pub unit: String,
}

fn default_unit() -> String {
    "pt".to_string()
}

// Payload for CMD_PARSE_TABLE
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseTablePayload {
    pub file_path: PathBuf,
    pub page_index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint_bbox: Option<BoundingBox>,
    #[serde(default = "default_confidence")]
    pub detection_confidence_threshold: f32,
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_confidence() -> f32 {
    0.7
}

fn default_language() -> String {
    "vie".to_string()
}

// Payload for RES_SUCCESS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessPayload<T> {
    pub req_id: Uuid,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

// Payload for RES_ERROR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPayload {
    pub req_id: Uuid,
    pub code: String,
    pub severity: ErrorSeverity,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_action: Option<String>,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_trace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorSeverity {
    Fatal,
    Error,
    Warning,
    Info,
}

// Payload for RES_PROGRESS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressPayload {
    pub req_id: Uuid,
    pub stage: String,
    pub current: u32,
    pub total: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eta_seconds: Option<u32>,
}

// Helper to create messages quickly
impl<T: Serialize> IpcMessage<T> {
    pub fn new(msg_type: MessageType, payload: T) -> Self {
        Self {
            protocol_v: "1.0.0".to_string(),
            msg_id: Uuid::new_v4(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            msg_type,
            payload,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

impl<'de, T: Deserialize<'de>> IpcMessage<T> {
    pub fn from_json(json_str: &'de str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let payload = HandshakeRequestPayload {
            rust_version: "1.83.0".to_string(),
            expected_protocol_v: "1.0.0".to_string(),
            capabilities_requested: vec!["ocr".to_string()],
        };

        let msg = IpcMessage::new(MessageType::CmdHandshake, payload);
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains("CMD_HANDSHAKE"));
        assert!(json.contains("protocol_v"));
        assert!(json.contains("1.0.0"));
    }
}
