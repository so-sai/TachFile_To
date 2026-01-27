---
description: Advanced debugging workflow for cleaning up Rust FFI linking errors on Windows (MSVC)
---

# Debugging Rust FFI Linking on Windows (MSVC)

Workflow này được đúc kết từ chiến dịch giải cứu `elite_pdf` (LNK2019, MuPDF 1.27.0). Sử dụng khi gặp lỗi "Unresolved External Symbol" dù đã link library.

## 1. Diagnose: Xác định thực trạng Symbol

Trước khi đổ lỗi cho code, hãy hỏi file `.lib`: Linker có *nhìn thấy* symbol không?

```powershell
# Tìm đường dẫn dumpbin (Visual Studio)
$dumpbin = (Get-ChildItem -Path "C:\Program Files\Microsoft Visual Studio" -Filter "dumpbin.exe" -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1).FullName

# 1. Kiểm tra exports của file .lib
& $dumpbin /EXPORTS path\to\library.lib | Select-String "symbol_name"

# 2. Nếu là Static Lib, check bảng Symbols (dữ liệu thô)
& $dumpbin /SYMBOLS path\to\library.lib | Select-String "symbol_name"
```

*   **Kết quả:**
    *   Thấy `Undef` hoặc không thấy gì: Symbol không tồn tại (do chưa compile vào, hoặc bị optimize mất).
    *   Thấy tên nhưng có ký tự lạ (`?function@@...`): Lỗi Name Mangling (C++ vs C). Cần thêm `extern "C"` trong C++ code hoặc dùng `#[link_name]` trong Rust.
    *   Thấy tên "sạch" nhưng Linker vẫn lỗi: **LTCG issue** (Xem bước 2).

## 2. Check: Whole Program Optimization (LTCG)

Nếu `dumpbin` thấy symbol nhưng Linker không thấy, khả năng cao file `.lib` chứa mã trung gian (Intermediate Language) thay vì Native Object Code.

```powershell
# Kiểm tra Header xem có flag LTCG không
& $dumpbin /HEADERS path\to\library.lib | Select-String "LTCG"
```

*   **Dấu hiệu:** Có dòng `Linker Time Code Generation` hoặc các Section lạ.
*   **Giải pháp:** Rebuild thư viện C/C++ với flag tắt LTCG.
    *   **MSBuild:** `/p:WholeProgramOptimization=false`
    *   **CMake:** `-DCPACK_C_FLAGS="/GL-" -DCPACK_CXX_FLAGS="/GL-"` (cho MSVC).

## 3. Verify: Architecture & Runtime

Đảm bảo sự đồng bộ giữa Rust và C Lib:

*   **Architecture:** x64 đi với x64. (Dùng `dumpbin /HEADERS` check `machine (x64)`).
*   **Runtime:** `MultiThreadedDLL (/MD)` (Rust default) vs `MultiThreaded (/MT)`.
    *   Nếu Lib build tĩnh (`/MT`), Linker sẽ la làng về conflict `LIBCMT` vs `MSVCRT`.
    *   **Fix:** Cấu hình lại `CFLAGS` của C Lib hoặc sửa `.cargo/config.toml` (ít khuyến khích).

## 4. The Surgical Fixes (Các đòn thế cụ thể)

### A. Missing Source File
Nếu symbol `fz_version` bị thiếu, check xem `version.c` có nằm trong file project (`.vcxproj` hoặc `CMakeLists.txt`) không.
*   *Action:* Grep source file trong thư mục build.

### B. Name Decoration (stdcall vs cdecl)
*   `_function@8` -> `stdcall` (Windows API hay dùng).
*   `_function` hoặc `function` -> `cdecl` (Standard C).
*   *Rust Fix:*
    ```rust
    extern "system" fn foo(); // stdcall
    extern "C" fn foo();      // cdecl
    ```

### C. Bypass / Workaround
Trong trường hợp khẩn cấp (Symbol là verify utility, không phải core logic), có thể hardcode hoặc reimplement phía Rust để bypass check.

---
**Lesson from Elite PDF:**
MuPDF 1.27.0 bật `/GL` (LTCG) mặc định trong Release build -> Rust linker mù tịt.
-> **Solution:** Rebuild với `/p:WholeProgramOptimization=false`.
