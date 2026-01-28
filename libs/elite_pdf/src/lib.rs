use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use std::ffi::{c_void, CString};
use std::panic::{self, AssertUnwindSafe};
use std::path::Path;
use std::ptr;
use std::sync::{Mutex, OnceLock};

mod ledger;
mod mupdf_text;
mod output;
mod sanitizer;
use output::OutputManager;

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
    fn fz_drop_context(ctx: *mut fz_context);
    fn fz_register_document_handlers(ctx: *mut fz_context);
    fn fz_open_document(ctx: *mut fz_context, filename: *const i8) -> *mut fz_document;
    fn fz_drop_document(ctx: *mut fz_context, doc: *mut fz_document);
    fn fz_count_pages(ctx: *mut fz_context, doc: *mut fz_document) -> i32;

    // Page & Pixmap API
    fn fz_load_page(ctx: *mut fz_context, doc: *mut fz_document, number: i32) -> *mut fz_page;
    fn fz_drop_page(ctx: *mut fz_context, page: *mut fz_page);
    fn fz_new_pixmap_from_page(
        ctx: *mut fz_context,
        page: *mut fz_page,
        ctm: fz_matrix,
        cs: *mut fz_colorspace,
        alpha: i32,
    ) -> *mut fz_pixmap;
    fn fz_drop_pixmap(ctx: *mut fz_context, pix: *mut fz_pixmap);
    fn fz_save_pixmap_as_png(ctx: *mut fz_context, pix: *mut fz_pixmap, filename: *const i8);
}

// =========================================================================
// 3. SAFE WRAPPER FOR ELITE PAGE (Needed for Mission 008)
// =========================================================================

pub struct ElitePage<'a> {
    ctx: crate::EliteContext,
    inner: *mut fz_page,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl ElitePage<'_> {
    pub fn from_document(doc: &EliteDocument, page_index: i32) -> Result<Self, String> {
        let master = get_master_context();
        let ctx = master.lock().unwrap();

        safe_ffi_call(|| {
            unsafe {
                let page = fz_load_page(ctx.as_ptr(), doc.inner, page_index);
                if page.is_null() {
                    return Err(PyRuntimeError::new_err("Failed to load page".to_string()));
                }

                Ok(Self {
                    ctx: ctx.clone(), // Clone context for ownership
                    inner: page,
                    _phantom: std::marker::PhantomData,
                })
            }
        })
        .map_err(|e| e.to_string())
    }

    pub fn extract_markdown(&self) -> Result<String, String> {
        let text_page = mupdf_text::EliteTextPage::from_page(self)?;
        text_page.to_markdown()
    }
}

impl<'a> Drop for ElitePage<'a> {
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
pub struct EliteContext {
    inner: *mut fz_context,
}

unsafe impl Send for EliteContext {}
unsafe impl Sync for EliteContext {}

impl Clone for EliteContext {
    fn clone(&self) -> Self {
        // Note: This is a shallow clone for thread safety
        // The actual context is shared and managed by MASTER_CONTEXT
        Self { inner: self.inner }
    }
}

const FZ_STORE_DEFAULT: usize = 256 << 20;

impl EliteContext {
    fn new_master() -> Option<Self> {
        let version = CString::new("1.27.0").unwrap();
        unsafe {
            let ptr =
                fz_new_context_imp(ptr::null(), ptr::null(), FZ_STORE_DEFAULT, version.as_ptr());
            if ptr.is_null() {
                None
            } else {
                fz_register_document_handlers(ptr);
                Some(Self { inner: ptr })
            }
        }
    }

    pub fn as_ptr(&self) -> *mut fz_context {
        self.inner
    }
}

impl Drop for EliteContext {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe {
                fz_drop_context(self.inner);
            }
            self.inner = ptr::null_mut();
        }
    }
}

static MASTER_CONTEXT: OnceLock<Mutex<EliteContext>> = OnceLock::new();

fn get_master_context() -> &'static Mutex<EliteContext> {
    MASTER_CONTEXT.get_or_init(|| {
        Mutex::new(EliteContext::new_master().expect("FATAL: MuPDF context initialization failed"))
    })
}

fn safe_ffi_call<F, T>(func: F) -> PyResult<T>
where
    F: FnOnce() -> PyResult<T> + std::panic::UnwindSafe,
{
    match panic::catch_unwind(AssertUnwindSafe(func)) {
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
    inner: *mut fz_document,
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
        let doc = safe_ffi_call({
            let filename_clone = filename.clone();
            move || {
                let c_filename = CString::new(filename_clone)
                    .map_err(|e| PyValueError::new_err(format!("Invalid path: {}", e)))?;

                let master = get_master_context();
                let ctx = master.lock().unwrap();

                unsafe {
                    let doc_ptr = fz_open_document(ctx.as_ptr(), c_filename.as_ptr());
                    if doc_ptr.is_null() {
                        return Err(PyRuntimeError::new_err(
                            "Failed to open document with MuPDF",
                        ));
                    }
                    Ok(Self { inner: doc_ptr })
                }
            }
        })?;

        // 4. Record to Ledger
        let page_count = doc.count_pages().unwrap_or(-1);
        let _ = ledger::record_ingestion(&filename, &hash, page_count, "SUCCESS");

        Ok(doc)
    }

    pub fn count_pages(&self) -> PyResult<i32> {
        safe_ffi_call(|| {
            let master = get_master_context();
            let ctx = master.lock().unwrap();
            unsafe { Ok(fz_count_pages(ctx.as_ptr(), self.inner)) }
        })
    }

    /// Extract structured text as Markdown from a page
    pub fn extract_page_markdown(&self, page_index: i32) -> PyResult<String> {
        safe_ffi_call(|| {
            let master = get_master_context();
            let ctx = master.lock().unwrap();

            // Create ElitePage wrapper
            let page = ElitePage::from_document(self, page_index)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to load page: {}", e)))?;

            // Extract Markdown
            let markdown = page.extract_markdown().map_err(|e| {
                PyRuntimeError::new_err(format!("Failed to extract markdown: {}", e))
            })?;

            Ok(markdown)
        })
    }

    /// Render một trang thành PNG và trích xuất JSON metadata
    pub fn process_page_evidence(
        &self,
        page_index: i32,
        output_dir: String,
    ) -> PyResult<(String, String)> {
        safe_ffi_call(move || {
            let master = get_master_context();
            let ctx_mutex = master.lock().unwrap();
            let ctx = ctx_mutex.as_ptr();

            unsafe {
                // 1. Load Page
                let page = fz_load_page(ctx, self.inner, page_index);
                if page.is_null() {
                    return Err(PyRuntimeError::new_err(format!(
                        "Failed to load page {}",
                        page_index
                    )));
                }

                // 2. Render PNG (Scale 2.0x for 144 DPI quality)
                let matrix = fz_matrix::scale(2.0);
                let pix = fz_new_pixmap_from_page(ctx, page, matrix, ptr::null_mut(), 0);
                if pix.is_null() {
                    fz_drop_page(ctx, page);
                    return Err(PyRuntimeError::new_err("Failed to create pixmap from page"));
                }

                let png_filename = format!("page_{}.png", page_index + 1);
                let png_path_buf = Path::new(&output_dir).join(&png_filename);
                let png_path_str = png_path_buf.to_string_lossy().to_string();
                let c_png_path = CString::new(png_path_str.clone()).unwrap();

                fz_save_pixmap_as_png(ctx, pix, c_png_path.as_ptr());

                // Cleanup Pixmap & Page
                fz_drop_pixmap(ctx, pix);
                fz_drop_page(ctx, page);

                // 3. Extract Meta (JSON Placeholder for now)
                let json_filename = format!("page_{}.json", page_index + 1);
                let json_path_buf = Path::new(&output_dir).join(&json_filename);
                let json_path_str = json_path_buf.to_string_lossy().to_string();

                let meta = serde_json::json!({
                    "page": page_index + 1,
                    "status": "PROCESSED",
                    "evidence_path": png_filename
                });

                std::fs::write(&json_path_buf, serde_json::to_string_pretty(&meta).unwrap())
                    .map_err(|e| PyRuntimeError::new_err(format!("Failed to write JSON: {}", e)))?;

                Ok((png_path_str, json_path_str))
            }
        })
    }

    /// Hàm mới: Xử lý Output tích hợp Cleanup
    pub fn process_output(&self, original_path: String) -> PyResult<String> {
        safe_ffi_call(move || {
            // 1. Khởi tạo OutputManager và tạo thư mục session
            let manager = OutputManager::new("output");
            let out_dir = manager
                .prepare_session_dir(&original_path)
                .map_err(|e| PyRuntimeError::new_err(format!("IO Error: {}", e)))?;

            let master = get_master_context();
            let ctx = master.lock().unwrap();
            let count = unsafe { fz_count_pages(ctx.as_ptr(), self.inner) };

            if count == 1 {
                // Case 1: 1 Trang -> Copy file nguồn làm bằng chứng
                manager
                    .handle_single_page(&original_path, &out_dir)
                    .map_err(|e| PyRuntimeError::new_err(format!("Copy failed: {}", e)))?;

                // Vẫn render page 1 để có PNG preview
                drop(ctx); // Release lock before calling self method if needed, but here we can call internal ffi
                self.process_page_evidence(0, out_dir.to_string_lossy().to_string())?;

                Ok(format!("Copied and rendered to: {}", out_dir.display()))
            } else {
                // Case 2: > 1 Trang -> Tạo bằng chứng trang đầu và ghi nhận
                drop(ctx);
                self.process_page_evidence(0, out_dir.to_string_lossy().to_string())?;

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
        if !self.inner.is_null() {
            let master = get_master_context();
            if let Ok(ctx) = master.lock() {
                unsafe {
                    fz_drop_document(ctx.as_ptr(), self.inner);
                }
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
