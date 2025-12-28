# LEGACY LESSONS - Python/Docling Era (Archived 2025-12-26)

These lessons belong to the defunct architecture using Python workers and are kept for historical context only. **DO NOT USE FOR V2.5+ DEVELOPMENT.**

---

## 1. Python 3.14 Alpha Compatibility
**Issue:** PyMuPDF 1.23.8 and Pillow 10.1.0 fail to install on Python 3.14 alpha due to missing pre-built wheels and C-extension compilation issues.
**Root Cause:** Alpha Python versions lack binary wheel support for many C-extension packages.
**Solution:** Use relaxed constraints in requirements.txt. Use lazy imports for optional dependencies.

---

## 2. Tauri Execution Context Detection (OS Error 267)
**Issue:** App fails to spawn Python worker with "The directory name is invalid (os error 267)".
**Root Cause:** Tauri apps execute from `ui/src-tauri/` directory, not project root.
**Solution:** Use dynamic path resolution by checking current directory and popping parents if in `src-tauri`.

---

## 4. Virtual Environment Path in Tauri
**Issue:** Handshake fails because system `python` is used instead of venv python.
**Root Cause:** Code used `"python"` string, which searches PATH instead of using project venv.
**Solution:** Use explicit paths to venv executables (e.g., `.venv/Scripts/python.exe` on Windows).

---

## 5. Stdio Buffer Flushing
**Issue:** IPC messages occasionally hang or arrive late.
**Root Cause:** OS buffers stdout, causing JSON messages to be delayed.
**Solution:** Always call `sys.stdout.flush()` after printing JSON messages in the Python worker.

---

## 8. Cache Eviction Not Implemented
**Issue:** `EvidenceCache.prune()` was a stub in the Python era.
**Impact:** Cache could grow unbounded.
**Planned Fix:** LRU eviction using SQLite timestamp ordering.

---
