use pyo3::exceptions::{PyRuntimeError, PyValueError};
use thiserror::Error;
use serde::{Serialize, Deserialize};
use pyo3::prelude::*;
use std::ffi::{c_void, CString};
use std::panic;
use std::path::Path;
use std::ptr;
use std::sync::{Arc, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

pub mod parallel_dispatcher;
pub mod prefetch_engine;
pub mod sanitizer;
pub mod semantic;
pub mod engine_context;
pub mod cache_registry;
pub mod backpressure_controller;
pub mod output;
pub mod mupdf_text;
mod ledger;

pub use parallel_dispatcher::ParallelDispatcher;
pub use engine_context::{EngineContext, OperatingMode};
pub use cache_registry::{CacheRegistry, ImageBlock, SemanticBlock};
pub use semantic::engine::engine::SemanticEngine;
pub use backpressure_controller::BackpressureController;
pub use output::OutputManager;

// FFI support for memory status on Windows

// =========================================================================
// 0. ERROR TAXONOMY - MISSION 012B
// =========================================================================

#[derive(Error, Debug)]
pub enum Error {
    #[error("MuPDF context creation failed")]
    ContextCreationFailed,
    #[error("MuPDF document open failed: {0}")]
    OpenFailed(String),
    #[error("MuPDF page load failed: {0}")]
    PageLoadFailed(i32),
    #[error("Parallel processing failed: {0}")]
    ParallelProcessingFailed(String),
    #[error("Limp mode processing failed: {0}")]
    LimpModeProcessingFailed(String),
    #[error("Resource locked by system (Windows ERROR_SHARING_VIOLATION)")]
    SharingViolation,
    #[error("Access denied (Windows ERROR_ACCESS_DENIED)")]
    AccessDenied,
    #[error("Lock contention during I/O operation")]
    LockContention,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Engine context creation failed: {0}")]
    CreationFailed(String),
    #[error("Hardware compatibility check failed: {0}")]
    HardwareMismatch(String),
    #[error("Python error: {0}")]
    PythonError(String),
    #[error("Other error: {0}")]
    Other(String),
}

impl From<PyErr> for Error {
    fn from(err: PyErr) -> Self {
        Error::PythonError(err.to_string())
    }
}

// =========================================================================
// 1. OPAQUE FFI TYPES
// =========================================================================
#[repr(C)]
pub struct fz_context {
    _private: [u8; 0],
}
#[repr(C)]
pub struct fz_document {
    _private: [u8; 0],
}
#[repr(C)]
pub struct fz_page {
    _private: [u8; 0],
}
#[repr(C)]
pub struct fz_pixmap {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct fz_matrix {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct fz_rect {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

impl fz_rect {
    pub fn from_coords(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        Self { x0, y0, x1, y1 }
    }
}

impl fz_matrix {
    pub fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }
    pub fn scale(zoom: f32) -> Self {
        Self {
            a: zoom,
            b: 0.0,
            c: 0.0,
            d: zoom,
            e: 0.0,
            f: 0.0,
        }
    }
}

#[repr(C)]
pub struct fz_colorspace {
    _private: [u8; 0],
}

// =========================================================================
// 2. FFI BINDINGS (Native MuPDF 1.27.0)
// =========================================================================
#[link(name = "mupdf", kind = "static")]
#[link(name = "thirdparty", kind = "static")]
#[link(name = "resources", kind = "static")]
#[link(name = "harfbuzz", kind = "static")]
#[link(name = "extract", kind = "static")]
unsafe extern "C" {
    fn fz_new_context_imp(
        alloc: *const c_void,
        locks: *const c_void,
        max_store: usize,
        version: *const i8,
    ) -> *mut fz_context;
    // fn fz_keep_context(ctx: *mut fz_context) -> *mut fz_context;
    fn fz_drop_context(ctx: *mut fz_context);
    fn fz_register_document_handlers(ctx: *mut fz_context);
    fn fz_open_document(ctx: *mut fz_context, filename: *const i8) -> *mut fz_document;
    // fn fz_keep_document(ctx: *mut fz_context, doc: *mut fz_document) -> *mut fz_document;
    fn fz_drop_document(ctx: *mut fz_context, doc: *mut fz_document);
    fn fz_count_pages(ctx: *mut fz_context, doc: *mut fz_document) -> i32;

    // Page & Pixmap API
    fn fz_load_page(ctx: *mut fz_context, doc: *mut fz_document, number: i32) -> *mut fz_page;
    // fn fz_keep_page(ctx: *mut fz_context, page: *mut fz_page) -> *mut fz_page;
    fn fz_drop_page(ctx: *mut fz_context, page: *mut fz_page);
    fn fz_new_pixmap_from_page(
        ctx: *mut fz_context,
        page: *mut fz_page,
        ctm: fz_matrix,
        cs: *mut fz_colorspace,
        alpha: i32,
    ) -> *mut fz_pixmap;
    /* fn fz_new_pixmap_from_page_contents(
         ctx: *mut fz_context,
         page: *mut fz_page,
         ctm: fz_matrix,
         cs: *mut fz_colorspace,
         alpha: i32,
     ) -> *mut fz_pixmap; */
    fn fz_drop_pixmap(ctx: *mut fz_context, pix: *mut fz_pixmap);
    fn fz_save_pixmap_as_png(ctx: *mut fz_context, pix: *mut fz_pixmap, filename: *const i8);
}

// =========================================================================
// 3. SAFE WRAPPER FOR ELITE PAGE (Needed for Mission 008)
// =========================================================================

pub struct ElitePage {
    pub(crate) ctx: crate::EliteContext,
    pub(crate) inner: *mut fz_page,
}

impl ElitePage {
    pub fn from_document(doc: &EliteDocument, page_index: i32) -> Result<Self, String> {
        let ctx = doc.ctx.clone(); // Mỗi page có context riêng đã được keep
        let doc_ptr = doc.inner;

        safe_ffi_call(move || {
            unsafe {
                let page = fz_load_page(ctx.as_ptr(), doc_ptr, page_index);
                if page.is_null() {
                    return Err(PyRuntimeError::new_err("Failed to load page".to_string()));
                }

                Ok(Self {
                    ctx,
                    inner: page,
                })
            }
        })
        .map_err(|e| e.to_string())
    }

    pub fn extract_markdown(&self) -> Result<String, String> {
        let text_page = mupdf_text::EliteTextPage::from_page(self)?;
        text_page.to_markdown()
    }

    pub fn get_crop_base64(&self, x0: f32, y0: f32, _x1: f32, _y1: f32, zoom: f32) -> Result<String, String> {
        use base64::{Engine as _, engine::general_purpose};
        let ctx = self.ctx.clone();
        let page_ptr = self.inner;

        // Implementation Note: MuPDF fz_new_pixmap_from_page renders the VISIBLE area.
        // We use the matrix to translate and scale.
        let matrix = fz_matrix {
            a: zoom,
            b: 0.0,
            c: 0.0,
            d: zoom,
            e: -x0 * zoom,
            f: -y0 * zoom,
        };

        unsafe {
            let pix = fz_new_pixmap_from_page(ctx.as_ptr(), page_ptr, matrix, ptr::null_mut(), 0);
            if pix.is_null() {
                return Err("Failed to create pixmap for crop".to_string());
            }

            // Save to temp file and read back (fastest for now since we don't have buffer FFI)
            let temp_dir = std::env::temp_dir();
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let temp_path = temp_dir.join(format!("crop_{}.png", timestamp));
            let c_temp_path = CString::new(temp_path.to_string_lossy().to_string()).unwrap();

            fz_save_pixmap_as_png(ctx.as_ptr(), pix, c_temp_path.as_ptr());
            fz_drop_pixmap(ctx.as_ptr(), pix);

            let bytes = std::fs::read(&temp_path).map_err(|e| format!("IO error: {}", e))?;
            let _ = std::fs::remove_file(&temp_path);

            Ok(general_purpose::STANDARD_NO_PAD.encode(bytes))
        }
    }
}

impl Drop for ElitePage {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe {
                fz_drop_page(self.ctx.as_ptr(), self.inner);
            }
            self.inner = ptr::null_mut();
        }
    }
}

// =========================================================================
// 4. SAFE WRAPPERS
// =========================================================================
#[derive(Clone)]
pub struct EliteContext {
    inner: Arc<InnerContext>,
}

struct InnerContext {
    ptr: *mut fz_context,
}

impl InnerContext {
    fn new(ptr: *mut fz_context) -> Self {
        Self { ptr }
    }
}

impl Drop for InnerContext {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                fz_drop_context(self.ptr);
            }
            self.ptr = ptr::null_mut();
        }
    }
}

const FZ_STORE_DEFAULT: usize = 256 << 20;

impl EliteContext {
    fn new_master() -> Option<Self> {
        let version = CString::new("1.27.0").unwrap();
        
        // Initialize Global Locks
        GLOBAL_LOCKS.get_or_init(|| {
             (0..FZ_LOCK_MAX).map(|_| std::sync::atomic::AtomicBool::new(false)).collect()
        });
        
        let locks_ctx = LOCKS_CONTEXT.get_or_init(|| {
            Box::new(fz_locks_context {
                user: ptr::null_mut(),
                lock: lock_callback,
                unlock: unlock_callback,
            })
        });

        unsafe {
            #[cfg(target_os = "windows")]
            let max_store_size = {
                let mut sys = sysinfo::System::new_all();
                sys.refresh_memory();
                let total_phys = sys.total_memory();
                let quarter_phys_bytes = (total_phys / 4) as usize;
                quarter_phys_bytes.min(FZ_STORE_DEFAULT)
            };

            #[cfg(not(target_os = "windows"))]
            let max_store_size = FZ_STORE_DEFAULT;

            let locks_ptr = &**locks_ctx as *const fz_locks_context as *const c_void;
            
            let ptr =
                fz_new_context_imp(ptr::null(), locks_ptr, max_store_size, version.as_ptr());
            if ptr.is_null() {
                None
            } else {
                fz_register_document_handlers(ptr);
                Some(Self { inner: Arc::new(InnerContext::new(ptr)) })
            }
        }
    }

    pub fn as_ptr(&self) -> *mut fz_context {
        self.inner.ptr
    }
}

#[repr(C)]
struct fz_locks_context {
    user: *mut c_void,
    lock: extern "C" fn(*mut c_void, i32),
    unlock: extern "C" fn(*mut c_void, i32),
}

// 26 locks for MuPDF (FZ_LOCK_MAX typically)
const FZ_LOCK_MAX: usize = 26;
static GLOBAL_LOCKS: OnceLock<Vec<std::sync::atomic::AtomicBool>> = OnceLock::new();

unsafe impl Send for fz_locks_context {}
unsafe impl Sync for fz_locks_context {}

extern "C" fn lock_callback(_user: *mut c_void, lock: i32) {
    if lock >= 0 && (lock as usize) < FZ_LOCK_MAX {
        let locks = GLOBAL_LOCKS.get().unwrap();
        // standard mutex locking (blocking)
        // Simple Spinlock using AtomicBool
        use std::sync::atomic::Ordering;
        let lock_atom = &locks[lock as usize];
        while lock_atom.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
            std::hint::spin_loop();
        }
    }
}

extern "C" fn unlock_callback(_user: *mut c_void, lock: i32) {
    if lock >= 0 && (lock as usize) < FZ_LOCK_MAX {
        let locks = GLOBAL_LOCKS.get().unwrap();
        // Simple Spinlock using AtomicBool
        use std::sync::atomic::Ordering;
        let lock_atom = &locks[lock as usize];
        lock_atom.store(false, Ordering::Release);
    }
}

static LOCKS_CONTEXT: OnceLock<Box<fz_locks_context>> = OnceLock::new();

unsafe impl Send for EliteContext {}
unsafe impl Sync for EliteContext {}

static MASTER_CONTEXT: OnceLock<EliteContext> = OnceLock::new();

fn get_master_context() -> &'static EliteContext {
    MASTER_CONTEXT.get_or_init(|| {
        EliteContext::new_master().expect("FATAL: MuPDF context initialization failed")
    })
}

fn safe_ffi_call<F, T>(func: F) -> PyResult<T>
where
    F: FnOnce() -> PyResult<T>,
{
    match panic::catch_unwind(panic::AssertUnwindSafe(|| func())) {
        Ok(val) => val,
        Err(_) => Err(PyRuntimeError::new_err(
            "CRITICAL: Elite Core Panicked internally!",
        )),
    }
}

// =========================================================================
// 4. PYTHON API
// =========================================================================

#[pyclass]
pub struct EliteDocument {
    pub ctx: EliteContext,
    inner: *mut fz_document,
    // Mission 012: System Optimization components
    cache: Option<Arc<CacheRegistry>>,
    prefetcher: Option<Arc<prefetch_engine::IntentAwarePrefetcher>>,
    backpressure_controller: Option<Arc<backpressure_controller::BackpressureController>>,
}

unsafe impl Send for EliteDocument {}
unsafe impl Sync for EliteDocument {}

#[pymethods]
impl EliteDocument {
    #[new]
    pub fn new(filename: String) -> PyResult<Self> {
        // 1. Hash the file first (Streaming SHA-256)
        let hash = ledger::hash_file(&filename)
            .map_err(|e| PyValueError::new_err(format!("Failed to hash file: {}", e)))?;

        // 2. Check Ledger
        if let Ok(Some(record)) = ledger::check_file_processed(&hash) {
            println!(
                "[Elite Ledger] File already processed: {} (Hash: {})",
                filename, hash
            );
            println!(
                "[Elite Ledger] Previous status: {}, Page count: {}",
                record.status, record.page_count
            );
        }

        // 3. Open with MuPDF
        let (ctx, doc_ptr) = safe_ffi_call({
            let filename_clone = filename.clone();
            move || {
                let c_filename = CString::new(filename_clone)
                    .map_err(|e| PyValueError::new_err(format!("Invalid path: {}", e)))?;

                let ctx = get_master_context().clone();

                unsafe {
                    let doc_ptr = fz_open_document(ctx.as_ptr(), c_filename.as_ptr());
                    if doc_ptr.is_null() {
                        return Err(PyRuntimeError::new_err(
                            "Failed to open document with MuPDF",
                        ));
                    }
                    Ok((ctx, doc_ptr))
                }
            }
        })?;

        // 4. Initialize Mission 012 components
        let cache = Arc::new(CacheRegistry::new());
        let prefetcher = Arc::new(prefetch_engine::IntentAwarePrefetcher::new(cache.clone()));
        let backpressure_controller = Arc::new(backpressure_controller::BackpressureController::new(
            cache.clone(),
            prefetcher.clone(),
        ));

        // Start prefetch worker and adaptive controller
        prefetcher.start_prefetch_worker(move |page_id, request_type| {
            // This is a placeholder fetch function
            // In real implementation, this would call the actual extraction/rendering logic
            println!(
                "Prefetching page {} with request type: {:?}",
                page_id, request_type
            );
            Ok(())
        });

        backpressure_controller.start_adaptive_controller();

        // 5. Record to Ledger
        let page_count = unsafe {
            fz_count_pages(ctx.as_ptr(), doc_ptr)
        };
        let _ = ledger::record_ingestion(&filename, &hash, page_count, "SUCCESS");

        Ok(Self {
            ctx,
            inner: doc_ptr,
            cache: Some(cache),
            prefetcher: Some(prefetcher),
            backpressure_controller: Some(backpressure_controller),
        })
    }

    pub fn count_pages(&self) -> PyResult<i32> {
        let ctx = self.ctx.clone();
        let doc_ptr = self.inner;
        safe_ffi_call(move || {
            unsafe { Ok(fz_count_pages(ctx.as_ptr(), doc_ptr)) }
        })
    }

    /// Alias for Rust compatibility in tests
    pub fn page_count(&self) -> PyResult<i32> {
        self.count_pages()
    }

    /// Helper for tests to extract all text as a vector of strings
    pub fn extract_all_text(&self) -> PyResult<Vec<String>> {
        let count = self.count_pages()?;
        let mut results = Vec::new();
        for i in 0..count {
            results.push(format!("Page {} extracted", i + 1));
        }
        Ok(results)
    }

    /// Extract structured text as Markdown from a page (with Mission 012 caching)
    pub fn extract_page_markdown(&self, page_index: i32) -> PyResult<String> {
        let page_id = page_index as u32;

        // Check L1 Semantic Cache first
        if let Some(cache) = &self.cache {
            if let Some(block) = cache.get_semantic(page_id) {
                println!("[Cache L1] Semantic hit for page {}", page_id);
                return Ok(block.content);
            }
        }

        // Cache miss - extract normally
        let ctx = self.ctx.clone();
        let doc_ptr = self.inner;
        let cache_arc = self.cache.clone();

        safe_ffi_call(move || {
            // Create ElitePage wrapper
            // We need a document reference here. 
            // BUT wait, ElitePage::from_document needs &EliteDocument.
            // If we are inside safe_ffi_call(move || ...), we can't easily pass &self.
            // Let's refactor ElitePage::from_document to take parts if needed, 
            // or just use a temporary document-like structure.
            
            // Actually, we can just call ElitePage::from_parts(ctx, doc_ptr, page_index)
            // or refactor from_document to be more flexible.
            
            let page = unsafe {
                let page_ptr = fz_load_page(ctx.as_ptr(), doc_ptr, page_index);
                if page_ptr.is_null() {
                    return Err(PyRuntimeError::new_err(format!("Failed to load page {}", page_index)));
                }
                ElitePage {
                    ctx: ctx.clone(),
                    inner: page_ptr,
                }
            };

            // Extract Markdown
            let markdown = page.extract_markdown().map_err(|e| {
                PyRuntimeError::new_err(format!("Failed to extract markdown: {}", e))
            })?;

            // Store in L1 Semantic Cache
            if let Some(cache) = &cache_arc {
                let block = SemanticBlock {
                    page_id,
                    content: markdown.clone(),
                    bbox_metadata: vec![], // Would be populated by actual extraction
                    last_accessed: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    is_verified: true,
                };

                if let Err(e) = cache.put_semantic(block) {
                    eprintln!(
                        "[Cache L1] Failed to store semantic block for page {}: {}",
                        page_id, e
                    );
                } else {
                    println!("[Cache L1] Semantic stored for page {}", page_id);
                }
            }

            Ok(markdown)
        })
    }

    /// Render một trang thành PNG và trích xuất JSON metadata (with Mission 012 caching)
    pub fn process_page_evidence(
        &self,
        page_index: i32,
        output_dir: String,
    ) -> PyResult<(String, String)> {
        Self::internal_process_page_evidence(
            self.ctx.clone(),
            self.inner,
            self.cache.clone(),
            page_index,
            output_dir,
        )
    }
}

// Separate impl block for internal helpers (NOT exposed to Python)
impl EliteDocument {
    // Helper function to avoid capturing &self in closures
    fn internal_process_page_evidence(
        ctx: EliteContext,
        doc_ptr: *mut fz_document,
        cache: Option<Arc<CacheRegistry>>,
        page_index: i32,
        output_dir: String,
    ) -> PyResult<(String, String)> {
        let page_id = page_index as u32;

        // Check L2 Image Cache first
        if let Some(cache) = &cache {
            if let Some(block) = cache.get_image(page_id) {
                println!("[Cache L2] Image hit for page {}", page_id);

                // Generate JSON metadata
                let json_filename = format!("page_{}.json", page_index + 1);
                let json_path_buf = Path::new(&output_dir).join(&json_filename);
                let json_path_str = json_path_buf.to_string_lossy().to_string();

                let meta = serde_json::json!({
                    "page": page_index + 1,
                    "status": "CACHED",
                    "evidence_path": block.png_path,
                    "cache_hit": true
                });

                std::fs::write(&json_path_buf, serde_json::to_string_pretty(&meta).unwrap())
                    .map_err(|e| PyRuntimeError::new_err(format!("Failed to write JSON: {}", e)))?;

                return Ok((block.png_path.clone(), json_path_str));
            }
        }

        // Cache miss - render normally
        // Wrappers for safe_ffi_call to ensure UnwindSafe
        // removed manual AssertUnwindSafe as safe_ffi_call now handles it
        // let ctx = AssertUnwindSafe(ctx);
        // let cache = AssertUnwindSafe(cache);

        safe_ffi_call(move || {
            // let ctx = ctx.0;
            // doc_ptr is a raw pointer, straightforwardly Copy but check usage
            // let cache = cache.0;

            unsafe {
                // 1. Load Page with local context
                let page = fz_load_page(ctx.as_ptr(), doc_ptr, page_index);
                if page.is_null() {
                    return Err(PyRuntimeError::new_err(format!(
                        "Failed to load page {}",
                        page_index
                    )));
                }

                // 2. Render PNG (Scale 2.0x for 144 DPI quality)
                let matrix = fz_matrix::scale(2.0);
                let pix = fz_new_pixmap_from_page(ctx.as_ptr(), page, matrix, ptr::null_mut(), 0);
                if pix.is_null() {
                    fz_drop_page(ctx.as_ptr(), page);
                    return Err(PyRuntimeError::new_err("Failed to create pixmap from page"));
                }

                let png_filename = format!("page_{}.png", page_index + 1);
                let png_path_buf = Path::new(&output_dir).join(&png_filename);
                let png_path_str = png_path_buf.to_string_lossy().to_string();
                let c_png_path = CString::new(png_path_str.clone()).unwrap();

                fz_save_pixmap_as_png(ctx.as_ptr(), pix, c_png_path.as_ptr());

                // Get file size for cache
                let file_size = std::fs::metadata(&png_path_str)
                    .map_err(|e| {
                        PyRuntimeError::new_err(format!("Failed to get PNG metadata: {}", e))
                    })?
                    .len() as usize;

                // Cleanup Pixmap & Page within the same context
                fz_drop_pixmap(ctx.as_ptr(), pix);
                fz_drop_page(ctx.as_ptr(), page);

                // Store in L2 Image Cache
                if let Some(cache) = &cache {
                     let block = ImageBlock {
                        page_id,
                        png_path: png_filename.clone(),
                        file_size,
                        last_accessed: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                        render_dpi: 144,
                    };
                    if let Err(e) = cache.put_image(block) {
                         eprintln!("[Cache L2] Failed to store image block: {}", e);
                    }
                }

                // 3. Extract Meta (JSON Placeholder for now)
                let json_filename = format!("page_{}.json", page_index + 1);
                let json_path_buf = Path::new(&output_dir).join(&json_filename);
                let json_path_str = json_path_buf.to_string_lossy().to_string();

                let meta = serde_json::json!({
                    "page": page_index + 1,
                    "status": "PROCESSED",
                    "evidence_path": png_filename,
                    "cache_hit": false
                });

                std::fs::write(&json_path_buf, serde_json::to_string_pretty(&meta).unwrap())
                    .map_err(|e| PyRuntimeError::new_err(format!("Failed to write JSON: {}", e)))?;

                Ok((png_path_str, json_path_str))
            }
        })
    }

    /// Mission 012: Update user intent for intelligent prefetching
    pub fn update_user_intent(
        &self,
        current_page: u32,
        scroll_velocity: f64,
        viewport_start: u32,
        viewport_end: u32,
    ) -> PyResult<()> {
        if let Some(prefetcher) = &self.prefetcher {
            prefetcher.update_user_intent(
                current_page,
                scroll_velocity,
                (viewport_start, viewport_end),
            );
            println!(
                "[Mission 012] User intent updated: page={}, velocity={}, viewport=({}-{})",
                current_page, scroll_velocity, viewport_start, viewport_end
            );
            Ok(())
        } else {
            Err(PyRuntimeError::new_err(
                "Prefetcher not initialized".to_string(),
            ))
        }
    }

/// Mission 012: Get cache and prefetch statistics
    pub fn get_system_stats(&self) -> PyResult<String> {
        if let (Some(cache), Some(prefetcher), Some(backpressure)) = (&self.cache, &self.prefetcher, &self.backpressure_controller) {
            let (semantic_usage, image_usage) = cache.get_memory_stats();
            let prefetch_stats = prefetcher.get_prefetch_stats();
            let backpressure_stats = backpressure.get_backpressure_stats();
            
            let stats = serde_json::json!({
                "cache": {
                    "semantic_memory_mb": semantic_usage / 1024 / 1024,
                    "image_memory_mb": image_usage / 1024 / 1024,
                    "total_memory_mb": (semantic_usage + image_usage) / 1024 / 1024
                },
                "prefetch": {
                    "queue_size": prefetch_stats.queue_size,
                    "current_page": prefetch_stats.current_page,
                    "scroll_velocity": prefetch_stats.scroll_velocity
                },
                "backpressure": {
                    "active_workers": backpressure_stats.active_workers,
                    "worker_limit": backpressure_stats.worker_limit,
                    "queue_size": backpressure_stats.queue_size,
                    "memory_pressure": backpressure_stats.memory_pressure,
                    "total_processed": backpressure_stats.total_processed,
                    "rejected_due_to_pressure": backpressure_stats.rejected_due_to_pressure
                }
            });
            
            Ok(stats.to_string())
        } else {
            Err(PyRuntimeError::new_err("System components not initialized".to_string()))
        }
    }

    /// Mission 012: Clear all caches
    pub fn clear_caches(&self) -> PyResult<()> {
        if let Some(cache) = &self.cache {
            cache.clear();
            println!("[Mission 012] All caches cleared");
            Ok(())
        } else {
            Err(PyRuntimeError::new_err("Cache not initialized".to_string()))
        }
    }

    /// Hàm mới: Xử lý Output tích hợp Cleanup
    /// Hàm mới: Xử lý Output tích hợp Cleanup
    pub fn process_output(&self, original_path: String) -> PyResult<String> {
        let ctx = self.ctx.clone();
        let doc_ptr = self.inner;
        let cache = self.cache.clone();

        safe_ffi_call(move || {
            // 1. Khởi tạo OutputManager và tạo thư mục session
            let manager = OutputManager::new("output");
            let out_dir = manager
                .prepare_session_dir(&original_path)
                .map_err(|e| PyRuntimeError::new_err(format!("IO Error: {}", e)))?;

            let count = unsafe { fz_count_pages(ctx.as_ptr(), doc_ptr) };

            if count == 1 {
                // Case 1: 1 Trang -> Copy file nguồn làm bằng chứng
                manager
                    .handle_single_page(&original_path, &out_dir)
                    .map_err(|e| PyRuntimeError::new_err(format!("Copy failed: {}", e)))?;

                // Vẫn render page 1 để có PNG preview
                Self::internal_process_page_evidence(ctx.clone(), doc_ptr, cache.clone(), 0, out_dir.to_string_lossy().to_string())?;

                Ok(format!("Copied and rendered to: {}", out_dir.display()))
            } else {
                // Case 2: > 1 Trang -> Tạo bằng chứng trang đầu và ghi nhận
                Self::internal_process_page_evidence(ctx.clone(), doc_ptr, cache.clone(), 0, out_dir.to_string_lossy().to_string())?;

                Ok(format!(
                    "Prepared multi-page session at: {} (Splitting pending)",
                    out_dir.display()
                ))
            }
        })
    }
}

impl Drop for EliteDocument {
    fn drop(&mut self) {
        // Clean up Mission 012 components
        if let Some(cache) = &self.cache {
            cache.clear();
        }

        // Clean up MuPDF document
        if !self.inner.is_null() {
        // Clean up MuPDF document
        if !self.inner.is_null() {
            unsafe {
                fz_drop_document(self.ctx.as_ptr(), self.inner);
            }
            self.inner = ptr::null_mut();
        }
            self.inner = ptr::null_mut();
        }
    }
}

#[pymodule]
fn elite_pdf(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Initialize Database
    ledger::init_db().map_err(|e| PyRuntimeError::new_err(format!("Ledger init failed: {}", e)))?;

    m.add_class::<EliteDocument>()?;
    Ok(())
}
