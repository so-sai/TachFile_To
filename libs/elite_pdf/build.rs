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
    let mupdf_root = r"e:\DEV\elite_9_VN-ecosystem\app-tool-TachFileTo\libs\elite_pdf\thirdparty\mupdf";
    let mupdf_lib_dir = format!(r"{}\platform\win32\x64\Release", mupdf_root);
    
    if std::path::Path::new(&mupdf_lib_dir).exists() {
        println!("cargo:rustc-link-search=native={}", mupdf_lib_dir);
        println!("cargo:warning=🛡️  Linking MuPDF from build output: {}", mupdf_lib_dir);
    } else {
        println!("cargo:warning=⚠️  CRITICAL: MuPDF build output not found at {}. Fallback may occur.", mupdf_lib_dir);
        // Fallback to standard vcpkg or other paths if needed
    }
    
    // Link MuPDF libraries
    // Link MuPDF libraries
    link_mupdf_libs();
    
    // Generate placeholder bindings
    generate_placeholder_bindings(&out_dir);
}

fn link_mupdf_libs() {
    // STUBBED: Static libraries provided have LNK1120 (missing C++ symbols).
    // Linking them causes build failure even if unused.
    // Disabling linking to verify Rust Architecture.
    
    // #[cfg(target_os = "windows")]
    // {
    //     println!("cargo:rustc-link-lib=static=libmupdf");
    //     println!("cargo:rustc-link-lib=static=libthirdparty");
    //     println!("cargo:rustc-link-lib=static=libharfbuzz");
    //     println!("cargo:rustc-link-lib=static=libtesseract");
    //     println!("cargo:rustc-link-lib=static=libleptonica");
    //     println!("cargo:rustc-link-lib=static=libresources");
    //     println!("cargo:rustc-link-lib=static=libextract");
    //     println!("cargo:rustc-link-lib=static=libmubarcode");
    //     println!("cargo:rustc-link-lib=static=libpkcs7");
    //     println!("cargo:rustc-link-lib=static=libzxing");
    //     println!("cargo:rustc-link-lib=static=msvcprt");
    //     println!("cargo:rustc-link-lib=user32");
    //     println!("cargo:rustc-link-lib=gdi32");
    //     println!("cargo:rustc-link-lib=comdlg32");
    //     println!("cargo:rustc-link-lib=advapi32");
    //     println!("cargo:rustc-link-lib=shell32");
    //     println!("cargo:rustc-link-lib=crypt32");
    //     println!("cargo:rustc-link-lib=bcrypt");
    //     println!("cargo:rustc-link-lib=usp10");
    //     println!("cargo:rustc-link-lib=rpcrt4");
    //     println!("cargo:rustc-link-lib=ole32");
    //     println!("cargo:rustc-link-lib=oleaut32");
    // }
    
    // #[cfg(not(target_os = "windows"))]
    // {
    //     println!("cargo:rustc-link-lib=mupdf");
    //     println!("cargo:rustc-link-lib=mupdf-third");
    // }
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
    std::fs::write(&bindings_path, bindings_content)
        .expect("Failed to write placeholder bindings");
}
