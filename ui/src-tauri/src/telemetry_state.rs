use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref FFI_LOCK_ACTIVE: AtomicBool = AtomicBool::new(false);
    pub static ref IS_LIMP_MODE: AtomicBool = AtomicBool::new(false);
    pub static ref JANITOR_STATUS: Mutex<String> = Mutex::new("IDLE".to_string());
}

pub fn set_ffi_lock(active: bool) {
    FFI_LOCK_ACTIVE.store(active, Ordering::Relaxed);
}

pub fn set_limp_mode(enabled: bool) {
    IS_LIMP_MODE.store(enabled, Ordering::Relaxed);
}

pub fn set_janitor_status(status: &str) {
    if let Ok(mut s) = JANITOR_STATUS.lock() {
        *s = status.to_string();
    }
}
