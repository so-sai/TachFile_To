// =========================================================================
// MISSION 011: ENGINECONTEXT - HARDWARE DETECTION & MODE LOCKING
// =========================================================================
// RAII-based root with Hardware probing and Operating Mode determination
// Tuân thủ Fail-Closed principle + Immutable Limp Mode

use crate::sanitizer::SimdSanitizer;
use pyo3::prelude::*;
use crate::Error;
use sysinfo::System;
use rand;

// =========================================================================
// 1. OPERATING MODE ENUM - IMMUTABLE AFTER CREATION
// =========================================================================

#[pyclass(skip_from_py_object)]
#[derive(Debug, Clone, serde::Serialize)]
pub enum FeatureType {
    Simd,
    ParallelExtraction,
    MemoryAwareCache,
    Prefetching,
    LedgerAudit,
}

#[pyclass(skip_from_py_object)]
#[derive(Debug, Clone, serde::Serialize)]
pub enum OperatingMode {
    /// Chế độ hiệu năng cao: Đầy đủ tài nguyên
    Normal {
        ram_gb: f32,
        logical_cores: usize,
        simd_enabled: bool,
    },
    /// Chế độ bảo trì: Giới hạn tính năng do thiếu tài nguyên hoặc lỗi phần cứng
    Limp {
        reason: String,
        remaining_features: Vec<String>,
    },
}

#[pyclass(skip_from_py_object)]
#[derive(Debug, Clone, serde::Serialize)]
pub struct TelemetryData {
    #[pyo3(get)]
    pub timestamp: u64,
    #[pyo3(get)]
    pub cpu_usage: f32,
    #[pyo3(get)]
    pub memory_free_mb: u64,
    #[pyo3(get)]
    pub mode: OperatingMode,
}

// =========================================================================
// 2. ENGINE CONTEXT - THE ROOT OF RAII
// =========================================================================

#[pyclass(skip_from_py_object)]
#[derive(Debug, Clone)]
pub struct EngineContext {
    /// Con trỏ gốc đến MuPDF context (unsafe raw pointer)
    pub ctx: *mut crate::fz_context,

    /// Trạng thái vận hành đã khóa (không thể thay đổi)
    pub mode: OperatingMode,

    /// Fingerprint phần cứng cho audit
    #[pyo3(get)]
    pub hardware_fingerprint: String,
}

unsafe impl Send for EngineContext {}
unsafe impl Sync for EngineContext {}

#[pymethods]
impl EngineContext {
    #[new]
    pub fn new() -> PyResult<Self> {
        println!("🛡️ TachFileTo: EngineContext initialization...");

        // 1. Thực hiện Hardware Probing
        let mode = Self::probe_hardware();

        // 2. Khởi tạo MuPDF Context theo mode
        let ctx = Self::create_mupdf_context(&mode).map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Context creation failed: {:?}", e))
        })?;

        // 3. Tạo hardware fingerprint (Mockup for Mission 011)
        let fingerprint = format!("ELITE-{:X}", rand::random::<u64>());

        println!("   🔍 Hardware fingerprint: {}", fingerprint);

        Ok(Self {
            ctx,
            mode,
            hardware_fingerprint: fingerprint,
        })
    }

    /// Trả về metadata vận hành hiện tại
    #[getter]
    pub fn get_mode(&self) -> OperatingMode {
        self.mode.clone()
    }

    /// Audit telemetry cho Tauri frontend
    pub fn get_telemetry(&self) -> TelemetryData {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        TelemetryData {
            timestamp: now,
            cpu_usage: 15.5, // Mockup (Sẽ tích hợp sysinfo ở Mission 012)
            memory_free_mb: 4096,
            mode: self.mode.clone(),
        }
    }
}

// =========================================================================
// 3. PRIVATE HARDWARE LOGIC
// =========================================================================

impl EngineContext {
    /// Thăm dò phần cứng để quyết định OperatingMode
    fn probe_hardware() -> OperatingMode {
        let mut sys = System::new_all();
        sys.refresh_all();

        let total_ram = sys.total_memory(); // in bytes
        let ram_gb = total_ram as f32 / (1024.0 * 1024.0 * 1024.0);
        let logical_cores = sys.cpus().len();

        println!("   RAM detected: {:.2} GB", ram_gb);
        println!("   Logical cores: {}", logical_cores);

        // Kiểm tra SIMD support
        let simd_supported = SimdSanitizer::is_x86_simd_supported();

        if ram_gb < 2.0 || logical_cores < 2 {
            let reason = if ram_gb < 2.0 {
                format!("Insufficient RAM: {:.2}GB < 2GB minimum", ram_gb)
            } else {
                format!("Insufficient cores: {} < 2 minimum", logical_cores)
            };
            println!("   ⚠️  Entering LIMP MODE: {}", reason);
            OperatingMode::Limp {
                reason,
                remaining_features: vec!["LedgerAudit".to_string()],
            }
        } else {
            println!("   ✅ Entering NORMAL MODE:");
            OperatingMode::Normal {
                ram_gb,
                logical_cores,
                simd_enabled: simd_supported,
            }
        }
    }

    /// Khởi tạo MuPDF context với Error handling
    fn create_mupdf_context(_mode: &OperatingMode) -> Result<*mut crate::fz_context, Error> {
        // We will delegate actual creation to fz_new_context_imp in lib.rs or future Master RAII
        // For current build purposes, we just return null if fail closed
        // In Mission 012, this will link to the real FFI
        Ok(std::ptr::null_mut())
    }
}

impl Drop for EngineContext {
    fn drop(&mut self) {
        if !self.ctx.is_null() {
            unsafe {
                crate::fz_drop_context(self.ctx);
            }
            self.ctx = std::ptr::null_mut();
        }
    }
}
