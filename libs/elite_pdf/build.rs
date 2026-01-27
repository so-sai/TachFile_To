//! Build script for Elite PDF
//!
//! This script handles linking with MuPDF library.
//! Fixed for Elite 9 Production Spec - Windows MSVC.

use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-env-changed=MUPDF_DIR");
    println!("cargo:rerun-if-env-changed=MUPDF_LIB_DIR");
    println!("cargo:rerun-if-changed=build.rs");

    // ELITE 9 WINDOWS STRATEGY - BUILD FROM SOURCE RESULT (Internal Secret)
    let mupdf_root =
        r"e:\DEV\elite_9_VN-ecosystem\app-tool-TachFileTo\libs\elite_pdf\thirdparty\mupdf";
    let mupdf_lib_dir = format!(r"{}\platform\win32\x64\Release", mupdf_root);

    // Phase 5: Link MuPDF Core Libraries
    println!("cargo:rustc-link-search=native={}", mupdf_lib_dir);
    
    // Core Libraries (Static)
    let mupdf_libs = [
        "mupdf", "thirdparty", "resources", "harfbuzz", 
        "extract", "pkcs7", "zxing", "leptonica", "tesseract"
    ];

    for lib in mupdf_libs {
        println!("cargo:rustc-link-lib=static={}", lib);
    }

    // System Libraries (Dynamics)
    let sys_libs = [
        "user32", "gdi32", "advapi32", "shell32", "ole32", "oleaut32", 
        "comdlg32", "crypt32", "msimg32", "windowscodecs", "winspool"
    ];
    for lib in sys_libs {
        println!("cargo:rustc-link-lib={}", lib);
    }

    println!("cargo:warning=🛡️  Phase 5: Full native linking activated");

    // Generate placeholder bindings
    generate_placeholder_bindings(&out_dir);
}

#[cfg(not(target_os = "windows"))]
fn link_mupdf_libs() {} // Stub for other platforms


#[cfg(not(target_os = "windows"))]
fn link_mupdf_libs() {
    println!("cargo:warning=⚠️  Non-Windows platform detected. MuPDF linking not configured.");
}

fn generate_placeholder_bindings(out_dir: &PathBuf) {
    let bindings_content = r#"
// Auto-generated placeholder bindings for MuPDF
#![allow(dead_code)]
#![allow(non_camel_case_types)]

pub const FZ_STORE_UNLIMITED: usize = 0;
pub type fz_context = std::ffi::c_void;
pub type fz_document = std::ffi::c_void;
pub type fz_page = std::ffi::c_void;
pub type fz_pixmap = std::ffi::c_void;
pub type fz_stext_page = std::ffi::c_void;
"#;

    let bindings_path = out_dir.join("mupdf_bindings.rs");
    std::fs::write(&bindings_path, bindings_content).expect("Failed to write placeholder bindings");
}
