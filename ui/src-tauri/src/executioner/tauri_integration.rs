/*
 * TAURI STARTUP INTEGRATION EXAMPLE - Phase 3: The Janitor
 * ==========================================================
 * 
 * CÁCH SỬ DỤNG:
 * Thêm code này vào tauri::Builder setup hook (trong main.rs hoặc lib.rs)
 * 
 * TIMING CỤC KỲ QUAN TRỌNG:
 * 1. Ledger PHẢI verify trước mọi thứ
 * 2. Janitor cleanup PHẢI xong trước UI interactive
 * 3. Nếu cleanup fail → startup FAIL (không che giấu lỗi)
 * 
 * CẢNH BÁO KỸ THUẬT:
 * - Nếu Ghost files > 10,000 → cleanup có thể mất 5-10 giây
 * - Cân nhắc chạy trên thread riêng + timeout
 * - Đừng bao giờ swallow errors (log chúng)
 */

use tauri::Manager;
use std::path::PathBuf;
use crate::executioner::{Janitor, SqliteLedger, LedgerBackend};
use crate::resource_court::CacheRegistry;

// ============================================================
// SETUP HOOK - Được gọi trước khi UI khởi động
// ============================================================

/// Hàm setup cho Tauri - chạy Janitor trước UI interactive
/// 
/// PHÁT HÀNH CHO: Tauri Builder
/// 
/// VÍ DỤ:
/// ```rust
/// tauri::Builder::default()
///     .setup(setup_janitor)
///     .manage(ExcelAppState::default())
///     // ... rest of builder
/// ```
#[cfg_attr(not(target_env = "msvc"), allow(dead_code))]
pub fn setup_janitor(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Lấy cache directory từ app config
    let cache_dir = app
        .path()
        .cache_dir()
        .map_err(|e| format!("Failed to get cache dir: {}", e))?;

    // Đảm bảo cache directory tồn tại
    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create cache dir: {}", e))?;

    // Mở hoặc tạo Ledger
    let ledger_path = cache_dir.join("janitor_ledger.db");
    let mut ledger = SqliteLedger::open(&ledger_path)
        .map_err(|e| format!("Failed to open Ledger: {}", e))?;

    // Tạo/load CacheRegistry
    // NOTE: Trong thực tế, bạn sẽ load từ một file hoặc state
    let registry = CacheRegistry::new();

    // Khởi tạo Janitor
    let janitor = Janitor::new(cache_dir.clone());

    // **QUÂN ĐÔI BƯỚC NÀY: Chạy cleanup**
    println!("[JANITOR] Starting Phase 3 cleanup...");

    match janitor.startup_cleanup(&mut ledger, &registry) {
        Ok(report) => {
            println!("[JANITOR] ✅ Cleanup completed successfully!");
            println!("[JANITOR] {}", report.summary());

            // Log tóm tắt cho audit trail
            if report.ghosts_deleted > 0 {
                println!(
                    "[JANITOR] Deleted {} ghost files",
                    report.ghosts_deleted
                );
            }

            if report.zombies_recovered > 0 {
                println!(
                    "[JANITOR] Recovered {} zombie warrants",
                    report.zombies_recovered
                );
            }

            // ✅ Nếu cleanup thành công → tiếp tục startup
            Ok(())
        }
        Err(e) => {
            // ❌ Nếu cleanup fail → **DỪNG NGAY**
            eprintln!("[JANITOR] ❌ STARTUP FAILED: {}", e);
            eprintln!("[JANITOR] Application cannot start with corrupted state!");

            // Trả về error → Tauri sẽ không khởi động app
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Janitor cleanup failed: {}", e),
            )))
        }
    }
}

// ============================================================
// ALTERNATIVE: Non-blocking startup (với timeout)
// ============================================================

/// Nếu bạn muốn cleanup chạy trong background với timeout
/// (tránh splash screen "treo")
/// 
/// CẢNH BÁO: Điều này CHỈ nên sử dụng nếu:
/// 1. Cache directory có QUẢN NHIỀU Ghost files (> 10,000)
/// 2. Bạn có cơ chế fallback (xử lý cleanup failure)
/// 3. UI đã có mechanism để wait for janitor complete
#[cfg_attr(not(target_env = "msvc"), allow(dead_code))]
pub fn setup_janitor_with_timeout(app: &mut tauri::App, timeout_secs: u64) -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Duration;
    use std::sync::Arc;

    let cache_dir = app
        .path()
        .cache_dir()
        .map_err(|e| format!("Failed to get cache dir: {}", e))?;

    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create cache dir: {}", e))?;

    let ledger_path = cache_dir.join("janitor_ledger.db");
    let mut ledger = SqliteLedger::open(&ledger_path)
        .map_err(|e| format!("Failed to open Ledger: {}", e))?;

    let registry = CacheRegistry::new();

    // Verify Ledger ngay lập tức (không block)
    ledger.verify_integrity()
        .map_err(|e| format!("Ledger corruption detected: {}", e))?;

    println!("[JANITOR] Ledger verified, starting cleanup in background...");

    // Spawn thread cho cleanup
    let janitor = Janitor::new(cache_dir.clone());
    std::thread::spawn(move || {
        match janitor.startup_cleanup(&mut ledger, &registry) {
            Ok(report) => {
                println!("[JANITOR] Background cleanup completed: {}", report.summary());
            }
            Err(e) => {
                eprintln!("[JANITOR] Background cleanup failed: {}", e);
                // Log to file for later analysis
            }
        }
    });

    // UI có thể start ngay, nhưng biết rằng cleanup đang chạy
    Ok(())
}

// ============================================================
// MAIN.RS INTEGRATION
// ============================================================

/*
THÊM VÀO main.rs hoặc lib.rs:

```rust
pub fn run() {
    tauri::Builder::default()
        .setup(setup_janitor)  // ← THÊM DÒNG NÀY
        .manage(ExcelAppState::default())
        .invoke_handler(tauri::generate_handler![
            // ... commands
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

HOẶC, nếu bạn muốn non-blocking:

```rust
pub fn run() {
    tauri::Builder::default()
        .setup(|app| setup_janitor_with_timeout(app, 30))  // ← 30 giây timeout
        .manage(ExcelAppState::default())
        // ... rest of builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```
*/

// ============================================================
// VERIFICATION CHECKLIST (AUDIT-GRADE)
// ============================================================

/*
✅ CHECKLIST TRƯỚC KHI SHIP:

1. Ledger Integrity
   - [ ] Ledger.verify_integrity() được gọi TRƯỚC cleanup
   - [ ] Nếu verify fail → startup FAIL (không skip)
   - [ ] Log message include "corruption detected"

2. Ghost File Detection
   - [ ] NamingContract luôn được áp dụng
   - [ ] Chỉ xóa files với TFT_ prefix
   - [ ] Alien files KHÔNG bao giờ bị xóa

3. Zombie Warrant Recovery
   - [ ] Tất cả PENDING warrants được hoàn tất
   - [ ] Mỗi hoàn tất được ghi vào Ledger
   - [ ] Nếu file đã gone → OK (idempotent)

4. Error Handling
   - [ ] Permission errors được log (không fail)
   - [ ] I/O errors được capture
   - [ ] Timeout được xử lý (nếu dùng threading)

5. Audit Trail
   - [ ] Janitor ghi tất cả actions vào Ledger
   - [ ] Report.summary() được in ra console/log
   - [ ] Nó có thể trace lại được cleanup decisions

6. Performance
   - [ ] Cleanup < 5 giây với < 1000 Ghost files
   - [ ] Cleanup < 30 giây với < 10,000 Ghost files
   - [ ] Nếu lâu hơn → consider multi-threaded scan

7. Testing
   - [ ] Test Ghost deletion (file created, then deleted)
   - [ ] Test Alien protection (user file NOT deleted)
   - [ ] Test Zombie recovery (PENDING warrant completion)
   - [ ] Test corruption handling (bad ledger → startup fail)
*/
