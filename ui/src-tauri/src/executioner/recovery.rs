/*
 * RECOVERY MODULE - PHASE 3: THE JANITOR (Bộ cảm biến Startup)
 * ==============================================================
 * 
 * Mục đích:
 * - Giải quyết asymmetry giữa Disk Reality và Ledger Truth
 * - Xóa Ghost files (TFT_ prefix nhưng không có trong Registry)
 * - Hoàn tất Zombie warrants (PENDING trạng thái)
 * - Đảm bảo sự im lặng sau cơn bão (Atomic cleanup)
 * 
 * Vai trò: Nhân viên vệ sinh với thẩm quyền pháp lý
 * - KHÔNG đưa ra quyết định mới
 * - KHÔNG sửa Ledger (chỉ ghi vào)
 * - KHÔNG suy đoán ý định của hệ thống
 * - CHỈ đối chiếu Reality ↔ Ledger
 * 
 * Chiến lược Lifetime:
 * - Janitor nhận &mut L (exclusive borrow)
 * - Không Clone Ledger hay Connection
 * - Sở hữu các error messages và cleanup records
 */

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::io;

use crate::executioner::api::{NamingContract, FileOrigin};
use crate::executioner::ledger::{LedgerBackend, ExecutionEventEntry, ExecutionResult};
use crate::resource_court::CacheRegistry;

// ============================================================
// JANITOR STRUCT - SINGLE RESPONSIBILITY
// ============================================================

/// Nhân viên vệ sinh có thẩm quyền pháp lý
/// 
/// Phải chạy ngay sau khởi động, trước UI interactive
/// - Xóa Ghost files (có TFT_ prefix nhưng không có trong Registry)
/// - Hoàn tất Zombie warrants (PENDING trạng thái chưa xong)
/// - Ghi log mọi hành động để audit
pub struct Janitor {
    cache_dir: PathBuf,
}

impl Janitor {
    /// Khởi tạo Janitor với đường dẫn cache directory
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    /// Khởi chạy quá trình dọn dẹp nguyên tử (ATOMIC CLEANUP)
    /// 
    /// Trình tự:
    /// 1. Verify Ledger integrity (nếu fail → startup FAIL)
    /// 2. Xử lý Zombies (hoàn tất PENDING warrants)
    /// 3. Quét disk cho Ghosts (TFT_ files không có trong Registry)
    /// 4. Xóa Ghosts và ghi log
    /// 
    /// Lưu ý kỷ thuật: Phải chạy trên luồng riêng hoặc có timeout
    /// để tránh ứng dụng "treo" ở splash screen
    pub fn startup_cleanup<L: LedgerBackend>(
        &self,
        ledger: &mut L,
        registry: &CacheRegistry,
    ) -> Result<JanitorReport, JanitorError> {
        let mut report = JanitorReport::new();

        // PHASE 1: Verify Ledger - Nếu fail, ứng dụng KHÔNG được khởi động
        ledger.verify_integrity().map_err(|e| {
            JanitorError::LedgerCorrupted(format!("Ledger integrity check failed: {:?}", e))
        })?;

        // PHASE 2: Xử lý Zombies (PENDING warrants)
        self.recover_zombies(ledger, &mut report)?;

        // PHASE 3: Tìm và xóa Ghosts
        self.find_and_purge_ghosts(ledger, registry, &mut report)?;

        Ok(report)
    }

    /// Hoàn tất các Zombie warrants (PENDING trạng thái)
    /// 
    /// Logic: Nếu warrant có PENDING trạng thái trong Ledger
    /// → Có nghĩa là quá trình thực thi bị gián đoạn
    /// → Cần hoàn tất việc xóa file khỏi disk
    fn recover_zombies<L: LedgerBackend>(
        &self,
        ledger: &mut L,
        report: &mut JanitorReport,
    ) -> Result<(), JanitorError> {
        // Lấy danh sách các warrants PENDING
        let pending = ledger
            .get_pending_warrants()
            .map_err(|e| JanitorError::LedgerQueryFailed(format!("Failed to get pending: {:?}", e)))?;

        for warrant in pending {
            // Mỗi warrant đại diện cho một file cần được xóa
            let target_path = &warrant.target_path;

            // Cố gắng xóa file nếu nó vẫn tồn tại
            let delete_result = self.attempt_hard_delete(target_path);

            // Ghi log kết quả vào Ledger
            let execution_result = match &delete_result {
                Ok(()) => {
                    report.zombies_recovered += 1;
                    ExecutionResult::Success
                }
                Err(JanitorError::FileNotFound(_)) => {
                    // File đã bị xóa rồi hoặc không bao giờ được tạo → OK
                    report.zombies_recovered += 1;
                    ExecutionResult::Success
                }
                Err(JanitorError::PermissionDenied(_)) => ExecutionResult::FailPermission,
                Err(JanitorError::IoError(_)) => ExecutionResult::FailIo,
                Err(JanitorError::FileLocked(_)) => ExecutionResult::FailLocked,
                Err(_) => ExecutionResult::FailIo,
            };

            // Ghi vào Ledger
            let event = ExecutionEventEntry {
                id: 0, // Auto-increment
                warrant_nonce: warrant.nonce.clone(),
                executed_at_unix: current_timestamp(),
                executor_id: "JANITOR_RECOVERY".to_string(),
                result: execution_result.as_str().to_string(),
                errno: extract_errno(&delete_result),
            };

            ledger.record_execution(&event).map_err(|e| {
                JanitorError::LedgerRecordFailed(format!(
                    "Failed to record recovery of warrant {}: {:?}",
                    warrant.nonce, e
                ))
            })?;
        }

        Ok(())
    }

    /// Tìm Ghost files (TFT_ prefix nhưng không có trong Registry)
    /// 
    /// Logic: Scan disk → classify với NamingContract
    /// → Nếu Ghost AND không có trong Registry → DELETE
    /// 
    /// Best Practice: Luôn lấy basename, không dùng fullpath để classify
    fn find_and_purge_ghosts<L: LedgerBackend>(
        &self,
        ledger: &mut L,
        registry: &CacheRegistry,
        report: &mut JanitorReport,
    ) -> Result<(), JanitorError> {
        // Nếu cache_dir không tồn tại, không có gì để dọn
        if !self.cache_dir.exists() {
            return Ok(());
        }

        // Lấy danh sách entries trong Registry (HashMap<file_id, CacheEntry>)
        let registry_entries = registry.entries();

        // Scan cache directory
        let entries = fs::read_dir(&self.cache_dir)
            .map_err(|e| JanitorError::ScanFailed(format!("Failed to scan cache dir: {}", e)))?;

        for entry_result in entries {
            let entry = entry_result
                .map_err(|e| JanitorError::ScanFailed(format!("Failed to read entry: {}", e)))?;

            let path = entry.path();

            // RULE: Luôn lấy basename → tránh lỗi với fullpath classification
            let file_name = match path.file_name() {
                Some(name) => match name.to_str() {
                    Some(s) => s,
                    None => {
                        // File name không phải UTF-8 → Alien (bỏ qua)
                        report.aliens_protected += 1;
                        continue;
                    }
                },
                None => {
                    // Không có file name (path error) → bỏ qua
                    continue;
                }
            };

            // Classify file
            let origin = NamingContract::classify(file_name);

            match origin {
                FileOrigin::Ghost => {
                    // Là Ghost → kiểm tra xem có trong Registry không
                    if !registry_entries.contains_key(file_name) {
                        // Ghost file không có trong Registry → DELETE
                        match self.attempt_hard_delete(path.to_str().unwrap_or(file_name)) {
                            Ok(()) => {
                                report.ghosts_deleted += 1;

                                // Ghi vào Ledger như một "internal cleanup"
                                let event = ExecutionEventEntry {
                                    id: 0,
                                    warrant_nonce: format!("GHOST_CLEANUP_{}", file_name),
                                    executed_at_unix: current_timestamp(),
                                    executor_id: "JANITOR_GHOST_CLEANUP".to_string(),
                                    result: ExecutionResult::Success.as_str().to_string(),
                                    errno: None,
                                };

                                let _ = ledger.record_execution(&event);
                            }
                            Err(e) => {
                                report.ghost_cleanup_errors += 1;
                                eprintln!("Failed to delete ghost file {}: {:?}", file_name, e);
                            }
                        }
                    } else {
                        // Ghost file HAS trong Registry → KHÔNG xóa
                        // (Nó vẫn được quản lý bởi Registry)
                        report.ghosts_protected += 1;
                    }
                }
                FileOrigin::Alien => {
                    // Không phải TFT file → KHÔNG CHẠM VÀO
                    report.aliens_protected += 1;
                }
            }
        }

        Ok(())
    }

    /// Cố gắng xóa hard-delete một file
    fn attempt_hard_delete(&self, path: &str) -> Result<(), JanitorError> {
        let path = Path::new(path);

        // Cố gắng xóa file
        fs::remove_file(path).map_err(|e| match e.kind() {
            io::ErrorKind::NotFound => JanitorError::FileNotFound(path.to_string_lossy().to_string()),
            io::ErrorKind::PermissionDenied => {
                JanitorError::PermissionDenied(path.to_string_lossy().to_string())
            }
            _ => JanitorError::IoError(format!("Failed to delete {}: {}", path.display(), e)),
        })?;

        Ok(())
    }
}

// ============================================================
// JANITOR REPORT - MỘT TỜ GIẤY CHỨNG THỰC
// ============================================================

/// Báo cáo dọn dẹp từ Janitor
/// "Lá phiếu xác nhận công việc"
#[derive(Debug, Clone)]
pub struct JanitorReport {
    /// Số lượng Zombie warrants đã được recover
    pub zombies_recovered: usize,
    /// Số lượng Ghost files đã bị xóa
    pub ghosts_deleted: usize,
    /// Số lượng Ghost files được bảo vệ (vì vẫn trong Registry)
    pub ghosts_protected: usize,
    /// Số lượng Alien files được bảo vệ (không xóa)
    pub aliens_protected: usize,
    /// Số lỗi trong quá trình xóa Ghost
    pub ghost_cleanup_errors: usize,
}

impl JanitorReport {
    pub fn new() -> Self {
        Self {
            zombies_recovered: 0,
            ghosts_deleted: 0,
            ghosts_protected: 0,
            aliens_protected: 0,
            ghost_cleanup_errors: 0,
        }
    }

    /// Kiểm tra xem cleanup có thành công hoàn toàn không
    pub fn is_successful(&self) -> bool {
        self.ghost_cleanup_errors == 0
    }

    /// Tóm tắt báo cáo
    pub fn summary(&self) -> String {
        format!(
            "Janitor Report: {} zombies recovered, {} ghosts deleted, {} protected, {} alien protected, {} errors",
            self.zombies_recovered,
            self.ghosts_deleted,
            self.ghosts_protected,
            self.aliens_protected,
            self.ghost_cleanup_errors
        )
    }
}

// ============================================================
// ERROR TYPES - PHÂN LOẠI RỦI RO
// ============================================================

#[derive(Debug)]
pub enum JanitorError {
    /// Ledger bị hỏng hoặc không consistent
    LedgerCorrupted(String),
    /// Không thể query Ledger
    LedgerQueryFailed(String),
    /// Không thể ghi vào Ledger
    LedgerRecordFailed(String),
    /// Không tìm thấy file
    FileNotFound(String),
    /// Permission denied
    PermissionDenied(String),
    /// Generic I/O error
    IoError(String),
    /// File bị lock
    FileLocked(String),
    /// Lỗi scan directory
    ScanFailed(String),
}

impl std::fmt::Display for JanitorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JanitorError::LedgerCorrupted(msg) => write!(f, "Ledger corrupted: {}", msg),
            JanitorError::LedgerQueryFailed(msg) => write!(f, "Ledger query failed: {}", msg),
            JanitorError::LedgerRecordFailed(msg) => write!(f, "Failed to record to ledger: {}", msg),
            JanitorError::FileNotFound(path) => write!(f, "File not found: {}", path),
            JanitorError::PermissionDenied(path) => write!(f, "Permission denied: {}", path),
            JanitorError::IoError(msg) => write!(f, "I/O error: {}", msg),
            JanitorError::FileLocked(path) => write!(f, "File locked: {}", path),
            JanitorError::ScanFailed(msg) => write!(f, "Scan failed: {}", msg),
        }
    }
}

impl std::error::Error for JanitorError {}

// ============================================================
// UTILITIES - HỖ TRỢ
// ============================================================

/// Lấy current timestamp (unix seconds)
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Trích xuất errno từ kết quả xóa file
fn extract_errno(result: &Result<(), JanitorError>) -> Option<i32> {
    match result {
        Err(JanitorError::PermissionDenied(_)) => Some(13), // EACCES
        Err(JanitorError::IoError(_)) => Some(5),           // EIO
        Err(JanitorError::FileLocked(_)) => Some(16),       // EBUSY
        _ => None,
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ghost_classification() {
        let ghost_file = "TFT_abc123_page_001_1609459200.tft_cache";
        assert_eq!(NamingContract::classify(ghost_file), FileOrigin::Ghost);
    }

    #[test]
    fn test_alien_classification() {
        let alien_file = "my_important_document.pdf";
        assert_eq!(NamingContract::classify(alien_file), FileOrigin::Alien);
    }

    #[test]
    fn test_janitor_report() {
        let mut report = JanitorReport::new();
        assert!(report.is_successful());

        report.ghost_cleanup_errors = 1;
        assert!(!report.is_successful());
    }

    #[test]
    fn test_janitor_report_summary() {
        let mut report = JanitorReport::new();
        report.zombies_recovered = 2;
        report.ghosts_deleted = 5;
        report.ghosts_protected = 3;
        report.aliens_protected = 1;

        let summary = report.summary();
        assert!(summary.contains("2 zombies recovered"));
        assert!(summary.contains("5 ghosts deleted"));
    }
}
