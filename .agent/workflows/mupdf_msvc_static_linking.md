---
description: How to link and initialize MuPDF Static Engine on Windows (MSVC)
---

# MuPDF MSVC Static Linking Doctrine

This workflow documents the hard-fought experience of linking MuPDF 1.27.0+ statically into a Rust `cdylib` on Windows.

## 1. Symbol Resolution Protocol
If `fz_new_context` is unresolved (LNK2019):
- **Problem**: MuPDF headers often inline `fz_new_context`, so the symbol isn't exported from the `.lib`.
- **Solution**: Call `fz_new_context_imp` directly.
- **FFI Signature**:
  ```rust
  fn fz_new_context_imp(alloc: *const c_void, locks: *const c_void, max_store: usize, version: *const i8) -> *mut fz_context;
  ```
- **Rule**: The `version` string MUST match the library version exactly (e.g., `"1.27.0"`), or `fz_new_context_imp` will return NULL with a version mismatch error.

## 2. Linker Configuration (`build.rs`)
- **Search Path**: Explicitly add the directory containing `.lib` files.
- **Library Names**: Standards vary, but using `println!("cargo:rustc-link-lib=static=libmupdf");` with files named `libmupdf.lib` is the target.
- **System Dependencies**: MUST link these Windows libs in order:
  `user32`, `gdi32`, `advapi32`, `shell32`, `ole32`, `oleaut32`, `comdlg32`, `crypt32`, `msimg32`, `windowscodecs`, `winspool`.

## 3. Workspace Profile Strategy
MuPDF static libs are often built without LTCG (`/GL-`). Rust's Link-Time Optimization (LTO) will clash with this.
- **Fix**: In the **ROOT** `Cargo.toml`:
  ```toml
  [profile.release]
  lto = false
  codegen-units = 1
  ```

## 4. Initialization & Safety
- **Context Management**: Do not attempt `fz_clone_context` unless you have implemented the `fz_locks_context` callbacks.
- **Elite Pattern**: Use a **Global Singleton Master Context** protected by a `Mutex` and `OnceLock`.
- **Thread Safety**: Wrap all FFI calls in a `Mutex` guard of the master context.

## 5. Verification
After building, the `.pyd` size should be **> 30MB**. Anything smaller indicates the engine was not linked and calls will fail at runtime.
