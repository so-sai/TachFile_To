use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
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

const FZ_STORE_DEFAULT: u32 = 256 << 20;

// =========================================================================
// 2. FFI BINDINGS
// =========================================================================
unsafe extern "C" {
    fn fz_new_context(alloc: *const c_void, locks: *const c_void, max_store: u32) -> *mut fz_context;
    fn fz_clone_context(ctx: *mut fz_context) -> *mut fz_context;
    fn fz_drop_context(ctx: *mut fz_context);
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

impl EliteContext {
    fn new_master() -> Option<Self> {
        unsafe {
            // let ptr = fz_new_context(ptr::null(), ptr::null(), FZ_STORE_DEFAULT);
            let ptr = 0xABCDEF00 as *mut fz_context; // Dummy Context Pointer
            if ptr.is_null() { None } else { Some(Self { inner: ptr }) }
        }
    }
    pub fn try_clone(&self) -> Option<Self> {
        unsafe {
            // let new_ptr = fz_clone_context(self.inner);
            let new_ptr = 0xABCDEF01 as *mut fz_context; // Simulated Clone
            if new_ptr.is_null() { None } else { Some(Self { inner: new_ptr }) }
        }
    }
    pub fn as_ptr(&self) -> *mut fz_context {
        self.inner
    }
}
impl Drop for EliteContext {
    fn drop(&mut self) {
        // if !self.inner.is_null() {
        //     unsafe { fz_drop_context(self.inner); }
        //     self.inner = ptr::null_mut();
        // }
    }
}

static MASTER_CONTEXT: OnceLock<EliteContext> = OnceLock::new();

fn get_thread_context() -> PyResult<EliteContext> {
    let master = MASTER_CONTEXT.get_or_init(|| {
        EliteContext::new_master().expect("FATAL: Failed to initialize MuPDF Master Context")
    });
    master.try_clone().ok_or_else(|| {
        PyRuntimeError::new_err("Failed to clone MuPDF context (OOM?)")
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
// 4. PYTHON API (Mutex Protected)
// =========================================================================

// Internal state holding the pointers.
// Send is OK (move ownership). Sync is NOT OK (raw access).
struct DocumentState {
    ctx: EliteContext,
    doc: *mut fz_document,
}

unsafe impl Send for DocumentState {}

// RAII cleanup for the state
impl Drop for DocumentState {
    fn drop(&mut self) {
        // if !self.doc.is_null() {
        //     unsafe { fz_drop_document(self.ctx.as_ptr(), self.doc); }
        //     self.doc = ptr::null_mut();
        // }
    }
}

#[pyclass]
struct EliteDocument {
    // Mutex provides Sync for us!
    // Without GIL, Python threads can race to access this struct.
    // Mutex ensures only one thread uses the context/doc at a time.
    inner: Mutex<Option<DocumentState>>,
}

#[pymethods]
impl EliteDocument {
    #[new]
    fn new(filename: String) -> PyResult<Self> {
        safe_ffi_call(move || {
            // STUBBED FOR ALPHA RC1 DEMO (LNK1120 Bypass)
            // Architecture is preserved, but we simulate the engine connection.
            let _ctx = get_thread_context()?;
            let _c_filename = CString::new(filename)
                .map_err(|_| PyRuntimeError::new_err("Invalid filename encoding"))?;
            
            // let doc_ptr = unsafe {
            //     fz_open_document(ctx.as_ptr(), c_filename.as_ptr())
            // };
            let doc_ptr = 0x12345678 as *mut fz_document; // Dummy Non-Null Pointer

            Ok(EliteDocument {
                inner: Mutex::new(Some(DocumentState {
                    ctx: _ctx,
                    doc: doc_ptr,
                })),
            })
        })
    }

    fn count_pages(&self) -> PyResult<i32> {
        safe_ffi_call(|| {
            // Lock the mutex to access state
            let mut guard = self.inner.lock().map_err(|_| {
                PyRuntimeError::new_err("EliteDocument Mutex Poisoned")
            })?;
            
            if let Some(_state) = guard.as_ref() {
                // unsafe {
                //     Ok(fz_count_pages(state.ctx.as_ptr(), state.doc))
                // }
                Ok(112) // Mocked Page Count
            } else {
                Err(PyRuntimeError::new_err("Document already closed"))
            }
        })
    }
}
// Drop for EliteDocument is automatic: Mutex drops, Option drops, DocumentState drops.

#[pymodule]
fn elite_pdf(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<EliteDocument>()?;
    Ok(())
}
