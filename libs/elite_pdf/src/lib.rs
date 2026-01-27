use pyo3::prelude::*;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use std::sync::{OnceLock, Mutex};
use std::ffi::{c_void, CString};
use std::ptr;
use std::panic::{self, AssertUnwindSafe};

// =========================================================================
// 1. OPAQUE FFI TYPES
// =========================================================================
#[repr(C)]
pub struct fz_context { _private: [u8; 0] }
#[repr(C)]
pub struct fz_document { _private: [u8; 0] }

// =========================================================================
// 2. FFI BINDINGS (Native MuPDF 1.27.0)
// =========================================================================
#[link(name = "mupdf", kind = "static")]
#[link(name = "thirdparty", kind = "static")]
#[link(name = "resources", kind = "static")]
#[link(name = "harfbuzz", kind = "static")]
#[link(name = "extract", kind = "static")]
unsafe extern "C" {
    fn fz_new_context_imp(alloc: *const c_void, locks: *const c_void, max_store: usize, version: *const i8) -> *mut fz_context;
    fn fz_drop_context(ctx: *mut fz_context);
    fn fz_register_document_handlers(ctx: *mut fz_context);
    fn fz_open_document(ctx: *mut fz_context, filename: *const i8) -> *mut fz_document;
    fn fz_drop_document(ctx: *mut fz_context, doc: *mut fz_document);
    fn fz_count_pages(ctx: *mut fz_context, doc: *mut fz_document) -> i32;
}

// =========================================================================
// 3. SAFE WRAPPERS
// =========================================================================
pub struct EliteContext {
    inner: *mut fz_context,
}

unsafe impl Send for EliteContext {}
unsafe impl Sync for EliteContext {}

const FZ_STORE_DEFAULT: usize = 256 << 20;

impl EliteContext {
    fn new_master() -> Option<Self> {
        let version = CString::new("1.27.0").unwrap();
        unsafe {
            let ptr = fz_new_context_imp(ptr::null(), ptr::null(), FZ_STORE_DEFAULT, version.as_ptr());
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
            unsafe { fz_drop_context(self.inner); }
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
        Err(_) => Err(PyRuntimeError::new_err("CRITICAL: Elite Core Panicked internally!")),
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
        safe_ffi_call(move || {
            let c_filename = CString::new(filename)
                .map_err(|e| PyValueError::new_err(format!("Invalid path: {}", e)))?;
            
            let master = get_master_context();
            let ctx = master.lock().unwrap();
            
            unsafe {
                let doc_ptr = fz_open_document(ctx.as_ptr(), c_filename.as_ptr());
                if doc_ptr.is_null() {
                    return Err(PyRuntimeError::new_err("Failed to open document with MuPDF"));
                }
                Ok(Self { inner: doc_ptr })
            }
        })
    }

    pub fn count_pages(&self) -> PyResult<i32> {
        safe_ffi_call(|| {
            let master = get_master_context();
            let ctx = master.lock().unwrap();
            unsafe {
                Ok(fz_count_pages(ctx.as_ptr(), self.inner))
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
    m.add_class::<EliteDocument>()?;
    Ok(())
}
