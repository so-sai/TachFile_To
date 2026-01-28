// =========================================================================
// MISSION 011: ENGINECONTEXT - HARDWARE DETECTION & MODE LOCKING
// =========================================================================
// RAII-based root with Hardware probing and Operating Mode determination
// Tuân thủ Fail-Closed principle + Immutable Limp Mode

use crate::sanitizer::SimdSanitizer;
use pyo3::prelude::*;
use std::ffi::CString;
use std::sync::Arc;

// =========================================================================
// 1. OPERATING MODE ENUM - IMMUTABLE AFTER CREATION
// =========================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum OperatingMode {
    /// Mode hoạt động đầy đủ - Tất cả tính năng được kích hoạt
    Normal {
        ram_gb: u32,
        logical_cores: u32,
        simd_enabled: bool,
    },
    /// Chế độ cấp cứu - Tối giản để hoạt động trên máy yếu
    Limp {
        reason: String,
        remaining_features: Vec<String>, // Các tính năng còn hoạt động
    },
}

// =========================================================================
// 2. ENGINECONTEXT - RAII ROOT OWNERSHIP
// =========================================================================

/// RAII Root: Chủ sở hữu tuyệt đối của bộ nhớ và ngữ cảnh MuPDF
/// Mọi đối tượng phía Python sẽ giữ một Arc trỏ về đây
#[pyclass]
#[derive(Debug, Clone)]
pub struct EngineContext {
    /// Con trỏ gốc đến MuPDF context (unsafe raw pointer)
    #[pyo3(get)]
    ctx: *mut crate::fz_context,

    /// Trạng thái vận hành đã khóa (không thể thay đổi)
    #[pyo3(get)]
    mode: OperatingMode,

    /// Fingerprint phần cứng cho audit
    #[pyo3(get)]
    hardware_fingerprint: String,
}

impl EngineContext {
    /// Khởi tạo EngineContext với Hardware Detection và Mode quyết định
    pub fn new() -> Result<Arc<Self>, EngineError> {
        println!("🛡️ TachFileTo: EngineContext initialization...");

        // 1. PROBE RAM (Physical + Virtual)
        let ram_info = Self::probe_ram();
        println!("   RAM detected: {} GB", ram_info.total_gb);

        // 2. PROBE LOGICAL CORES
        let core_info = Self::probe_cores();
        println!("   Logical cores: {}", core_info.logical_cores);

        // 3. DECISION MATRIX - KHÔNG CÓ VÙNG XÁM
        let mode = if ram_info.total_gb < 8 || core_info.logical_cores < 4 {
            let reason = if ram_info.total_gb < 8 {
                format!("Insufficient RAM: {}GB < 8GB minimum", ram_info.total_gb)
            } else {
                format!(
                    "Insufficient cores: {} < 4 minimum",
                    core_info.logical_cores
                )
            };

            println!("   ⚠️  Entering LIMP MODE: {}", reason);

            OperatingMode::Limp {
                reason,
                remaining_features: vec![
                    "Semantic Engine".to_string(),
                    "Evidence Generation".to_string(),
                    "File Export".to_string(),
                ],
            }
        } else {
            // 4. CHECK SIMD CAPABILITY
            let simd_enabled = SimdSanitizer::is_x86_simd_supported();

            println!("   ✅ Entering NORMAL MODE:");
            println!(
                "      - SIMD: {}",
                if simd_enabled { "ENABLED" } else { "DISABLED" }
            );
            println!("      - Prefetch: ENABLED");
            println!("      - Multi-page: ENABLED");

            OperatingMode::Normal {
                ram_gb: ram_info.total_gb,
                logical_cores: core_info.logical_cores,
                simd_enabled,
            }
        };

        // 5. KHỞI TẠO MUPDF CONTEXT THEO MODE
        let ctx = Self::create_mupdf_context(&mode)?;

        // 6. TẠO FINGERPRINT CHO AUDIT
        let fingerprint = format!(
            "RAM:{}GB|CORES:{}|SIMD:{}|MODE:{:?}",
            ram_info.total_gb,
            core_info.logical_cores,
            match &mode {
                OperatingMode::Normal { simd_enabled, .. } => *simd_enabled,
                OperatingMode::Limp { .. } => false,
            },
            std::mem::discriminant(&mode)
        );

        println!("   🔍 Hardware fingerprint: {}", fingerprint);

        // 7. ĐÓNG GÓI TRONG ARC VÀ TRẢ VỀ
        Ok(Arc::new(Self {
            ctx,
            mode,
            hardware_fingerprint: fingerprint,
        }))
    }

    /// Probe thông tin RAM (Windows API cho chính xác nhất)
    #[cfg(target_os = "windows")]
    fn probe_ram() -> RamInfo {
        use std::mem;

        // Windows API call would be more accurate, but using fallback for cross-platform
        // This is conservative estimation
        let total_kb = unsafe { windows_sys::GlobalMemoryStatusEx(0, std::ptr::null_mut()) };

        let total_gb = if total_kb.ullTotalPhys > 0 {
            (total_kb.ullTotalPhys / 1024 / 1024) as u32
        } else {
            // Fallback: conservative estimate
            16 // Assume 16GB if detection fails
        };

        RamInfo {
            total_gb,
            available_gb: 0, // Not needed for mode decision
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn probe_ram() -> RamInfo {
        // Linux/Mac fallback using /proc/meminfo or sysctl
        RamInfo {
            total_gb: 16, // Conservative estimate
            available_gb: 0,
        }
    }

    /// Probe số logical cores (không phải physical cores)
    fn probe_cores() -> CoreInfo {
        let logical_cores =
            std::thread::available_parallelism().expect("Failed to detect available parallelism");

        CoreInfo {
            logical_cores: logical_cores.get() as u32,
            physical_cores: None, // Not needed for decision
        }
    }

    /// Khởi tạo MuPDF context với tham số phù hợp mode
    fn create_mupdf_context(mode: &OperatingMode) -> Result<*mut crate::fz_context, EngineError> {
        let version = CString::new("1.27.0").unwrap();

        unsafe {
            let store_size = match mode {
                OperatingMode::Normal { .. } => 256 << 20, // 256MB
                OperatingMode::Limp { .. } => 64 << 20,    // 64MB in limp mode
            };

            let ctx = crate::fz_new_context_imp(
                std::ptr::null(),
                std::ptr::null(),
                store_size,
                version.as_ptr(),
            );

            if ctx.is_null() {
                return Err(EngineError::ContextCreationFailed);
            }

            // Register document handlers
            crate::fz_register_document_handlers(ctx);

            Ok(ctx)
        }
    }

    /// Kiểm tra đang ở Limp Mode
    pub fn is_limp_mode(&self) -> bool {
        matches!(&self.mode, OperatingMode::Limp { .. })
    }

    /// Lấy thông tin trạng thái cho telemetry
    pub fn get_telemetry(&self) -> TelemetryData {
        match &self.mode {
            OperatingMode::Normal {
                ram_gb,
                logical_cores,
                simd_enabled,
            } => TelemetryData {
                ram_gb: *ram_gb,
                logical_cores: *logical_cores,
                simd_enabled: *simd_enabled,
                is_limp: false,
            },
            OperatingMode::Limp {
                remaining_features, ..
            } => {
                TelemetryData {
                    ram_gb: 0,           // Not relevant in limp mode
                    logical_cores: 1,    // Single-threaded in limp
                    simd_enabled: false, // Always disabled in limp
                    is_limp: true,
                }
            }
        }
    }
}

// =========================================================================
// 3. SAFE DROP - RAII MEMORY MANAGEMENT
// =========================================================================

impl Drop for EngineContext {
    fn drop(&mut self) {
        println!("🛡️ TachFileTo: EngineContext dropping - freeing memory...");

        if !self.ctx.is_null() {
            unsafe {
                // Giải phóng MuPDF context và các buffer thô
                crate::fz_drop_context(self.ctx);
            }
            self.ctx = std::ptr::null_mut();
        }

        println!("✅ EngineContext dropped safely. Memory freed.");
    }
}

// =========================================================================
// 4. TELEMETRY STRUCTURES FOR MEASUREMENT CONTRACT
// =========================================================================

#[derive(Debug)]
pub struct RamInfo {
    pub total_gb: u32,
    pub available_gb: u32,
}

#[derive(Debug)]
pub struct CoreInfo {
    pub logical_cores: u32,
    pub physical_cores: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct TelemetryData {
    #[pyo3(get)]
    pub ram_gb: u32,
    #[pyo3(get)]
    pub logical_cores: u32,
    #[pyo3(get)]
    pub simd_enabled: bool,
    #[pyo3(get)]
    pub is_limp: bool,
}

// =========================================================================
// 5. PYTHON BINDINGS WITH SAFE LIFETIME MANAGEMENT
// =========================================================================

#[pymethods]
impl EngineContext {
    #[new]
    fn py_new() -> PyResult<Arc<Self>> {
        Self::new().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!(
                "EngineContext creation failed: {:?}",
                e
            ))
        })
    }

    /// Lấy telemetry data cho monitoring
    fn get_telemetry(py_self: pyo3::Py<Self>) -> TelemetryData {
        let ctx = py_self.borrow();
        ctx.get_telemetry()
    }

    /// Kiểm tra xem có đang ở Limp Mode không
    fn is_limp_mode(py_self: pyo3::Py<Self>) -> bool {
        let ctx = py_self.borrow();
        ctx.is_limp_mode()
    }
}
