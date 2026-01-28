use chrono::Local;
use std::fs::{self, create_dir_all, read_dir, remove_dir_all};
use std::io;
use std::path::{Path, PathBuf};

const MAX_KEEP_SESSIONS: usize = 10;

pub struct OutputManager {
    base_dir: PathBuf,
}

impl OutputManager {
    pub fn new(base_dir: &str) -> Self {
        let base = PathBuf::from(base_dir);
        if !base.exists() {
            let _ = create_dir_all(&base);
        }
        Self { base_dir: base }
    }

    /// Chuẩn bị thư mục session mới và thực hiện dọn dẹp các session cũ
    pub fn prepare_session_dir(&self, original_filename: &str) -> io::Result<PathBuf> {
        // Thực hiện dọn dẹp trước khi tạo mới
        self.cleanup_old_sessions()?;

        let now = Local::now();
        let timestamp = now.format("%Y%m%d_%H%M%S").to_string();

        // Sanitize tên file để dùng làm tên thư mục
        let path = Path::new(original_filename);
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown_file");

        let safe_stem = stem.replace(['\\', '/', ':', '*', '?', '"', '<', '>', '|', ' '], "_");

        let dir_name = format!("{}_{}", timestamp, safe_stem);
        let session_path = self.base_dir.join(dir_name);

        if !session_path.exists() {
            create_dir_all(&session_path)?;
        }

        Ok(session_path)
    }

    /// Giữ lại tối đa MAX_KEEP_SESSIONS thư mục mới nhất
    fn cleanup_old_sessions(&self) -> io::Result<()> {
        if !self.base_dir.exists() {
            return Ok(());
        }

        let mut sessions: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();

        for entry in read_dir(&self.base_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Ok(metadata) = fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        sessions.push((path, modified));
                    }
                }
            }
        }

        // Sắp xếp: Mới nhất trước (Reverse time)
        sessions.sort_by(|a, b| b.1.cmp(&a.1));

        // Giữ lại tối đa MAX_KEEP_SESSIONS - 1 thư mục cũ để sau khi tạo cái mới sẽ là MAX_KEEP_SESSIONS
        if sessions.len() >= MAX_KEEP_SESSIONS {
            // Ví dụ: MAX = 10. Nếu len = 10, skip(9) xóa phần tử thứ 10. Còn 9. Sau đó tạo 1 -> 10.
            for (path, _) in sessions.iter().skip(MAX_KEEP_SESSIONS - 1) {
                if let Err(e) = remove_dir_all(path) {
                    eprintln!(
                        "[Elite Output] Failed to clean old session {}: {}",
                        path.display(),
                        e
                    );
                } else {
                    println!("[Elite Output] Cleaned old session: {}", path.display());
                }
            }
        }

        Ok(())
    }

    /// Xử lý trường hợp 1 trang: Copy nguyên file gốc vào thư mục output
    pub fn handle_single_page(&self, src_path: &str, output_dir: &Path) -> io::Result<String> {
        let src = Path::new(src_path);
        let file_name = src
            .file_name()
            .ok_or(io::Error::new(io::ErrorKind::InvalidInput, "No filename"))?;
        let dest = output_dir.join(file_name);

        fs::copy(src, &dest)?;

        Ok(dest.to_string_lossy().to_string())
    }
}
