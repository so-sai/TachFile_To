//! IPC Protocol Types for TachFileTo
//!
//! This is the SOURCE OF TRUTH for the entire system.
//! Any changes here must be reflected in:
//! - backend/app/protocol.py (Python)
//! - src/types/ipc.ts (TypeScript)

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ==================== ENUMS ====================

/// Request priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Immediate,  // User clicked, needs <500ms response
    Normal,     // User hovered, needs <2s response
    Background, // Prefetch, can wait
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}

/// Error types that can occur during evidence extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
    FileNotFound,
    PageOutOfRange,
    MemoryExhausted,
    TimeoutExceeded,
    ParsingFailed,
    WorkerUnavailable,
}

// ==================== REQUEST TYPES ====================

/// Evidence extraction request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceRequest {
    pub request_id: String,
    pub file_path: PathBuf,
    pub page_index: usize,
    pub bbox: [f32; 4], // [x, y, width, height] in PDF coords
    pub dpi: u16,
    #[serde(default)]
    pub priority: Priority,
}

/// Command sent to Python worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerCommand {
    pub id: String,
    pub cmd: String,
    pub payload: serde_json::Value,
    #[serde(default)]
    pub priority: Priority,
}

// ==================== RESPONSE TYPES ====================

/// Successful evidence extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceSuccess {
    pub request_id: String,
    pub data_base64: String,
    pub mime_type: String,
    pub dimensions: (u32, u32),
    pub is_cache_hit: bool,
}

/// Pending response (request queued)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidencePending {
    pub request_id: String,
    pub queue_position: usize,
    pub estimated_wait_ms: u64,
}

/// Error response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceError {
    pub request_id: String,
    pub error_type: ErrorType,
    pub message: String,
    pub retry_after_ms: Option<u64>,
}

/// Combined response type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum EvidenceResponse {
    Success(EvidenceSuccess),
    Pending(EvidencePending),
    Failed(EvidenceError),
}

impl EvidenceResponse {
    pub fn success(
        request_id: String,
        data_base64: String,
        mime_type: String,
        dimensions: (u32, u32),
        is_cache_hit: bool,
    ) -> Self {
        Self::Success(EvidenceSuccess {
            request_id,
            data_base64,
            mime_type,
            dimensions,
            is_cache_hit,
        })
    }

    pub fn pending(request_id: String, queue_position: usize, estimated_wait_ms: u64) -> Self {
        Self::Pending(EvidencePending {
            request_id,
            queue_position,
            estimated_wait_ms,
        })
    }

    pub fn error(
        request_id: String,
        error_type: ErrorType,
        message: String,
        retry_after_ms: Option<u64>,
    ) -> Self {
        Self::Failed(EvidenceError {
            request_id,
            error_type,
            message,
            retry_after_ms,
        })
    }
}

// ==================== WORKER RESPONSE ====================

/// Response format from Python worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerResponse {
    pub req_id: String,
    pub status: String,
    pub data: Option<serde_json::Value>,
    pub error: Option<WorkerErrorDetail>,
    pub perf: Option<PerformanceMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerErrorDetail {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
    pub traceback: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub duration_ms: f64,
    pub peak_ram_mb: Option<f64>,
}

// ==================== LIFECYCLE MESSAGES ====================

/// Ready signal from Python worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerReadyMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub event: String,
    pub version: String,
    pub pid: u32,
    pub capabilities: Vec<String>,
    pub worker_id: String,
}

// ==================== HEALTH CHECK ====================

/// Health report for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthReport {
    pub status: String,
    pub metrics: HealthMetrics,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthMetrics {
    pub total_requests: u64,
    pub cache_hit_rate: f64,
    pub avg_response_time_ms: f64,
    pub memory_usage_mb: f64,
    pub queue_depth: usize,
    pub error_rate: f64,
}

impl Default for HealthMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            cache_hit_rate: 0.0,
            avg_response_time_ms: 0.0,
            memory_usage_mb: 0.0,
            queue_depth: 0,
            error_rate: 0.0,
        }
    }
}

// ==================== CACHE KEY ====================

/// Key for cache lookups
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CacheKey {
    pub file_hash: String,
    pub page_num: usize,
    pub dpi: u16,
    pub bbox_hash: String,
}

impl CacheKey {
    pub fn new(file_hash: &str, page_num: usize, dpi: u16, bbox: &[f32; 4]) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        for val in bbox {
            val.to_bits().hash(&mut hasher);
        }
        let bbox_hash = format!("{:x}", hasher.finish());

        Self {
            file_hash: file_hash.to_string(),
            page_num,
            dpi,
            bbox_hash,
        }
    }

    pub fn to_string_key(&self) -> String {
        format!(
            "{}:{}:{}:{}",
            self.file_hash, self.page_num, self.dpi, self.bbox_hash
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        let key1 = CacheKey::new("abc123", 5, 150, &[100.0, 200.0, 50.0, 30.0]);
        let key2 = CacheKey::new("abc123", 5, 150, &[100.0, 200.0, 50.0, 30.0]);
        let key3 = CacheKey::new("abc123", 5, 150, &[100.0, 200.0, 50.0, 31.0]);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_response_serialization() {
        let response = EvidenceResponse::success(
            "req-123".to_string(),
            "base64data".to_string(),
            "image/jpeg".to_string(),
            (100, 100),
            false,
        );

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("success"));
    }
}
