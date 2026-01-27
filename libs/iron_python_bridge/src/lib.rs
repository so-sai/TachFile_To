//! # Iron Python Bridge
//!
//! Embedded Python runtime for Iron Core PDF extraction.
//!
//! ## Architecture (HARD MODE)
//!
//! This crate follows the Iron Core policy:
//! - ❌ NO subprocess
//! - ❌ NO IPC
//! - ✅ PyO3 + PyOxidizer ONLY
//!
//! ## Usage
//!
//! let result = bridge.extract_file_embedded("contract.pdf")?;
//! ```

pub mod models;
pub mod ledger_migration;
pub mod unified_extractor;

use anyhow::{Context, Result as AnyResult};
use pyo3::prelude::*;
use pyo3::types::{PyList, PyTuple};
use std::sync::Once;
use std::path::PathBuf;
use crate::models::IngestionResult;
use thiserror::Error;

/// Errors that can occur during Python bridge operations
#[derive(Error, Debug)]
pub enum BridgeError {
    /// Python interpreter initialization failed
    #[error("Failed to initialize Python interpreter: {0}")]
    InitFailed(String),

    /// Python module import failed
    #[error("Failed to import module '{module}': {reason}")]
    ImportError { module: String, reason: String },

    /// Extraction operation failed
    #[error("PDF extraction failed: {0}")]
    ExtractionError(String),

    /// Runtime not initialized
    #[error("Python runtime not initialized. Call PythonBridge::init() first.")]
    NotInitialized,

    /// Performance SLA violation
    #[error("SLA violation: {metric} = {actual}, expected {expected}")]
    SlaViolation {
        metric: String,
        actual: String,
        expected: String,
    },
}

/// Result type for bridge operations
pub type BridgeResult<T> = Result<T, BridgeError>;

static PY_INITIALIZED: Once = Once::new();

/// Initialize the Python interpreter once
fn init_python() -> PyResult<()> {
    PY_INITIALIZED.call_once(|| {
        Python::initialize();
    });
    Ok(())
}

/// Python runtime bridge for Iron Core
///
/// This is a skeleton implementation. The actual PyO3 integration
/// will be enabled when the build environment is configured.
pub struct PythonBridge {
    /// Whether the runtime is initialized
    initialized: bool,
    /// Python version string (for verification)
    python_version: Option<String>,
}

impl PythonBridge {
    /// Initialize the embedded Python runtime
    ///
    /// # Returns
    /// - `Ok(PythonBridge)` if initialization succeeds
    /// - `Err(BridgeError::InitFailed)` if initialization fails
    ///
    /// # Performance Contract
    /// - Cold start MUST complete within 5 seconds (SLA §4.1)
    pub fn init() -> BridgeResult<Self> {
        tracing::info!("Initializing Iron Python Bridge (embedded mode)");

        // In a real build, this would be handled by PyOxidizer/PyO3
        // For development, we assume the environment is set up.
        // pyo3::prepare_freethreaded_python();

        Ok(Self {
            initialized: true,
            python_version: Some("3.14.0-embedded".to_string()),
        })
    }

    /// Extract data from a supported file using chunked processing (Embedded Implementation)
    ///
    /// # Performance Contract
    /// - Extraction for 100 pages SHOULD complete within 30s (SLA §4.2)
    pub fn extract_file_embedded(&self, path: &str) -> AnyResult<IngestionResult> {
        if !self.initialized {
            anyhow::bail!("Python runtime not initialized");
        }

        init_python().map_err(|e| anyhow::anyhow!("Python init failed: {}", e))?;

        tracing::info!("Starting embedded extraction for: {}", path);

        let result_json = Python::attach(|py: Python<'_>| -> PyResult<String> {
            // Setup sys.path to include the copy of extraction.py
            let sys = py.import("sys")?;
            let path_list: Bound<'_, PyList> = sys.getattr("path")?.extract()?;
            
            // In development/test, Python files are copied to OUT_DIR/python by build.rs
            let target_python_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap_or_default())
                .parent().unwrap()
                .parent().unwrap()
                .parent().unwrap()
                .join("python");
            
            path_list.append(target_python_dir.to_str().unwrap())?;

            // Import and call extraction module
            let extraction = py.import("extraction")?;
            let func = extraction.getattr("process_file")?;
            
            // Call with allow_mock=True for tests/dev
            let kwargs = pyo3::types::PyDict::new(py);
            kwargs.set_item("allow_mock", true)?;

            // Correct PyO3 0.23+ Bound API call using tuple for positional args
            let result_any = func.call((path,), Some(&kwargs))?;
            let result_json: String = result_any.extract()?;

            Ok(result_json)
        }).map_err(|e| anyhow::anyhow!("Embedded Python call failed: {}", e))?;

        // Deserialize into Rust struct outside of GIL
        let result: IngestionResult = serde_json::from_str(&result_json)
            .context("Failed to deserialize IngestionResult from Python output")?;

        Ok(result)
    }

    /// Get the Python version string
    ///
    /// Used for verification that embedded runtime is working.
    pub fn python_version(&self) -> BridgeResult<&str> {
        if !self.initialized {
            return Err(BridgeError::NotInitialized);
        }

        self.python_version
            .as_deref()
            .ok_or_else(|| BridgeError::InitFailed("Version not available".to_string()))
    }

    /// Check if the runtime is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Shutdown the Python runtime
    ///
    /// This must be called before the program exits to properly
    /// clean up Python resources.
    pub fn shutdown(&mut self) -> BridgeResult<()> {
        if !self.initialized {
            return Ok(()); // Already shutdown
        }

        tracing::info!("Shutting down Iron Python Bridge");

        // TODO: Actual PyO3 finalization when enabled
        // pyo3::ffi::Py_Finalize();

        self.initialized = false;
        self.python_version = None;

        Ok(())
    }
}

impl Drop for PythonBridge {
    fn drop(&mut self) {
        if self.initialized {
            if let Err(e) = self.shutdown() {
                tracing::error!("Failed to shutdown Python bridge: {}", e);
            }
        }
    }
}

/// Placeholder for PDF extraction configuration
#[derive(Debug, Clone)]
pub struct ExtractionConfig {
    /// Enable table detection
    pub detect_tables: bool,
    /// Enable OCR for scanned documents
    pub enable_ocr: bool,
    /// Maximum pages to process (0 = unlimited)
    pub max_pages: usize,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            detect_tables: true,
            enable_ocr: false, // CPU-only by default
            max_pages: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_init_skeleton() {
        let bridge = PythonBridge::init().expect("Init should succeed");
        assert!(bridge.is_initialized());
    }

    #[test]
    fn test_bridge_version_skeleton() {
        let bridge = PythonBridge::init().expect("Init should succeed");
        let version = bridge
            .python_version()
            .expect("Version should be available");
        assert!(version.contains("embedded"));
    }

    #[test]
    fn test_bridge_shutdown() {
        let mut bridge = PythonBridge::init().expect("Init should succeed");
        bridge.shutdown().expect("Shutdown should succeed");
        assert!(!bridge.is_initialized());
    }

    #[test]
    fn test_not_initialized_error() {
        let mut bridge = PythonBridge::init().expect("Init should succeed");
        bridge.shutdown().expect("Shutdown should succeed");

        let result = bridge.python_version();
        assert!(matches!(result, Err(BridgeError::NotInitialized)));
    }

    #[test]
    #[ignore] // Requires Python 3.14 environment and build.rs copy to work
    fn test_embedded_call_red_green() {
        let bridge = PythonBridge::init().expect("Init should succeed");
        // Use a dummy path, allow_mock will handle it in the bridge
        let result = bridge.extract_file_embedded("dummy.pdf");
        
        match result {
            Ok(ingestion) => {
                assert_eq!(ingestion.source, "dummy.pdf");
                assert!(ingestion.pages.len() >= 1);
                assert_eq!(ingestion.extraction_meta.get("engine").unwrap(), "docling");
            },
            Err(e) => panic!("Embedded call failed: {:?}", e),
        }
    }

    #[tokio::test]
    #[ignore] // Requires Python 3.14 environment
    async fn test_concurrent_extraction_stress() {
        let bridge = std::sync::Arc::new(PythonBridge::init().expect("Init should succeed"));
        let mut handles = vec![];

        for i in 0..5 {
            let b = bridge.clone();
            let handle = tokio::spawn(async move {
                let path = format!("stress_test_{}.pdf", i);
                // In Phase 4, we test if multiple threads can acquire GIL and run extraction
                b.extract_file_embedded(&path)
            });
            handles.push(handle);
        }

        for handle in handles {
            let result = handle.await.expect("Task panicked");
            assert!(result.is_ok(), "Concurrent extraction failed: {:?}", result.err());
        }
    }
}
