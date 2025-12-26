# LESSONS LEARNED - TachFileTo Development

**Appendix C: Implementation Lessons**  
**Last Updated:** 2025-12-25

---

## 1. Python 3.14 Alpha Compatibility

**Issue:** PyMuPDF 1.23.8 and Pillow 10.1.0 fail to install on Python 3.14 alpha due to missing pre-built wheels and C-extension compilation issues.

**Root Cause:** Alpha Python versions lack binary wheel support for many C-extension packages.

**Solution:**
```python
# requirements.txt - Use relaxed constraints
PyMuPDF  # Instead of PyMuPDF==1.23.8
Pillow   # Instead of Pillow==10.1.0
```

**Best Practice:** Use lazy imports to allow worker startup even if optional dependencies fail:
```python
try:
    import fitz  # PyMuPDF
except ImportError:
    fitz = None
    log.warning("PyMuPDF not available, evidence extraction disabled")
```

---

## 2. Tauri Execution Context Detection (OS Error 267)

**Issue:** App fails to spawn Python worker with "The directory name is invalid (os error 267)".

**Root Cause:** Tauri apps execute from `ui/src-tauri/` directory, not project root. Code tried to access `backend/` from wrong location.

**Solution:**
```rust
// manager.rs - Dynamic path resolution
let mut backend_dir = std::env::current_dir()?;

if backend_dir.ends_with("src-tauri") {
    backend_dir.pop(); // Remove src-tauri
    backend_dir.pop(); // Remove ui
} else if backend_dir.ends_with("ui") {
    backend_dir.pop(); // Remove ui
}

backend_dir.push("backend");
```

**Best Practice:** Always use dynamic path resolution in multi-directory workspaces. Never assume execution directory.

---

## 3. TailwindCSS v4 Breaking Change

**Issue:** Vite dev server fails with "Unknown at rule @tailwind" error.

**Root Cause:** Tailwind v4 requires `@tailwindcss/postcss` plugin instead of legacy `tailwindcss` plugin.

**Solution:**
```bash
npm install -D @tailwindcss/postcss
```

```javascript
// postcss.config.js
export default {
  plugins: {
    '@tailwindcss/postcss': {},  // New plugin
    autoprefixer: {},
  },
}
```

**Best Practice:** Check migration guides when upgrading major versions of CSS frameworks.

---

## 4. Virtual Environment Path in Tauri

**Issue:** Handshake fails because system `python` is used instead of venv python.

**Root Cause:** Code used `"python"` string, which searches PATH instead of using project venv.

**Solution:**
```rust
// lib.rs - Use venv python
let python_path = if cfg!(windows) {
    "../backend/.venv/Scripts/python.exe"
} else {
    "../backend/.venv/bin/python"
};
```

**Best Practice:** Always use explicit paths to virtual environment executables in production apps.

---

## 5. Stdio Buffer Flushing

**Issue:** IPC messages occasionally hang or arrive late.

**Root Cause:** OS buffers stdout, causing JSON messages to be delayed.

**Solution:**
```python
# main.py - Always flush after writing
print(json.dumps(message))
sys.stdout.flush()  # CRITICAL
```

**Best Practice:** For stdio-based IPC, always flush after each message to prevent buffer deadlocks.

---

## 6. Import Error vs Function Name Mismatch

**Issue:** Rust compilation fails with "unresolved import `fix_vietnamese_text`".

**Root Cause:** Documentation and code used different function names. Actual function is `detect_and_convert`.

**Solution:**
```rust
// lib.rs - Use correct function name
use tachfileto_core::text::detect_and_convert;

fn convert_legacy_text(text: String) -> String {
    let (_encoding, converted) = detect_and_convert(&text);
    converted
}
```

**Best Practice:** Keep documentation in sync with code. Use automated doc generation tools when possible.

---

## 7. Timeout Mismatch

**Issue:** Documentation specifies 3-second timeout, but code uses 2 seconds.

**Location:** 
- Doc: Section 6, line 321
- Code: `ui/src-tauri/src/lib.rs` line 45

**Resolution:** Updated code to match documentation:
```rust
tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
```

**Best Practice:** Use constants for timeout values to ensure consistency:
```rust
const WORKER_INIT_TIMEOUT_SECS: u64 = 3;
```

---

## 8. Cache Eviction Not Implemented

**Issue:** `EvidenceCache.prune()` is a stub, no actual eviction happens.

**Impact:** Cache can grow unbounded, consuming all disk space.

**Current State:**
```python
def prune(self, max_size_mb: int = 500, max_age_days: int = 30):
    # Implementation skipped for MVP
    pass
```

**Planned Fix (Phase 3):**
```python
def prune(self, max_size_mb: int = 500):
    # Get total cache size
    cursor.execute("SELECT SUM(LENGTH(image_data)) FROM evidence_cache")
    total_bytes = cursor.fetchone()[0] or 0
    
    if total_bytes > max_size_mb * 1024 * 1024:
        # Delete oldest entries
        cursor.execute("""
            DELETE FROM evidence_cache 
            WHERE id IN (
                SELECT id FROM evidence_cache 
                ORDER BY last_accessed ASC 
                LIMIT 100
            )
        """)
```

**Best Practice:** Implement resource limits before deploying to production.

---

## Summary Table

| Lesson | Severity | Status | Phase |
|--------|----------|--------|-------|
| Python 3.14 compatibility | High | ✅ Fixed | Phase 2 |
| Tauri path detection | Critical | ✅ Fixed | Phase 2 |
| TailwindCSS v4 | Medium | ✅ Fixed | Phase 2 |
| Venv python path | High | ✅ Fixed | Phase 2 |
| Stdio flushing | Medium | ✅ Fixed | Phase 1 |
| Function name mismatch | Medium | ✅ Fixed | Phase 2 |
| Timeout mismatch | Low | ⚠️ Documented | Phase 2 |
| Cache eviction | High | ❌ Planned | Phase 3 |

---

**Next Steps:**
1. Implement cache size limits and LRU eviction (Phase 3)
2. Add memory monitoring and quota system (Phase 3)
3. Implement fallback strategies for evidence extraction (Phase 4)
