# 📄 SPECIFICATION: PYTHON WORKER ARCHITECTURE (v1.0)

**Filename:** `docs/specs/backend_worker_v1.md`  
**Role:** The Heavy Lifter (Cơ bắp)  
**Responsibility:** CPU-bound tasks (OCR, PDF Parsing, Image Cropping)

---

## 1. ARCHITECTURE OVERVIEW

The Python Worker operates as a **Stateless Subprocess** managed by the Rust Core.
It follows the **"Let it crash"** philosophy: If it gets stuck or leaks memory, Rust kills it and spawns a new one.

```mermaid
graph TD
    Rust[Rust Core - Parent] -- stdin JSON Line --> Python[Python Worker - Child]
    Python -- stdout JSON Line --> Rust
    Python -- stderr --> LogFile
    
    subgraph Python Process
        MainLoop[Async Event Loop]
        ThreadPool[Executor - Max 1]
        Watchdog[Memory Monitor Thread]
        Docling[Docling Engine]
    end
    
    Watchdog -- RAM over 1GB --> Python : sys.exit 1
```

---

## 2. COMMUNICATION PROTOCOL (IPC)

We use **Standard I/O (stdio)** pipes.

* **Format:** JSON Lines (`ndjson`). Each line is a complete message.
* **Encoding:** UTF-8.
* **NewLine:** `\n` is the delimiter.

### 2.1. Handshake (Startup)

When Rust spawns Python, Python must perform a self-check and report readiness.

**Python -> Rust (Ready Signal):**
```json
{
  "type": "lifecycle",
  "event": "ready",
  "version": "0.1.0",
  "pid": 12345,
  "capabilities": ["docling", "mmap"]
}
```

### 2.2. Command Request (Rust -> Python)
```json
{
  "id": "req_uuid_v4",
  "cmd": "extract_evidence",
  "payload": {
    "file_path": "C:\\Projects\\Data\\plan.pdf",
    "page_index": 5,
    "bbox": [100.0, 200.0, 50.0, 50.0],
    "dpi": 150
  }
}
```

### 2.3. Command Response (Python -> Rust)
```json
{
  "req_id": "req_uuid_v4",
  "status": "success",
  "data": {
    "base64": "/9j/4AAQSkZJRg...",
    "width": 100,
    "height": 100
  },
  "perf": {
    "duration_ms": 150,
    "peak_ram_mb": 450.5
  }
}
```

---

## 3. CORE COMPONENTS

### 3.1. Main Loop (`app/main.py`)

* **AsyncIO based**: Uses `sys.stdin.readline()` in a non-blocking way.
* **Signal Handling**: Catches `SIGINT` / `SIGTERM` for graceful shutdown.
* **Router**: Dispatches commands to specific engines (`extract_evidence`, `parse_table`, etc.).

### 3.2. Engine Wrapper (`app/engine/extractor.py`)

* **Docling Integration**:
  * **Global Instance**: Docling model is loaded **once** at startup (expensive init).
  * **Thread Lock**: Since Docling/PyTorch is not thread-safe, we use a `Lock` or `ThreadPoolExecutor(max_workers=1)`.
  * **Timeout**: Every operation is wrapped in `asyncio.wait_for(timeout=5.0)`.

### 3.3. Memory Watchdog (`app/engine/memory_monitor.py`)

* **Background Thread**: Runs every 1 second.
* **Logic**:
  1. Check `psutil.Process().memory_info().rss`.
  2. If `RSS > 1GB` (Soft Limit): Trigger `gc.collect()`.
  3. If `RSS > 1.5GB` (Hard Limit): Send "suicide warning" to Rust, then `sys.exit(137)`.

---

## 4. ERROR HANDLING STRATEGY

The worker must **never** hang. It must always reply, even if it crashes.

| Scenario | Worker Action | Rust Action |
| --- | --- | --- |
| **PDF Corrupt** | Catch exception, return `status: "error"` | Display error placeholder |
| **Docling Hang** | `asyncio.TimeoutError` triggers after 5s | Retry once, then fail |
| **OOM (Out of Memory)** | Watchdog kills process | Detect exit code, restart Worker |
| **Unknown Panic** | Write traceback to `stderr` | Log error, restart Worker |

---

## 5. DEPENDENCIES & ENVIRONMENT

**Requirements (`requirements.txt`):**
```text
docling>=1.0.0,<2.0.0
pydantic>=2.0.0,<3.0.0
psutil>=5.9.0,<6.0.0
Pillow>=10.0.0,<11.0.0
```

**Environment Variables:**
* `TACH_WORKER_RAM_LIMIT_MB`: Default 1024.
* `TACH_DOCLING_THREADS`: Default 4 (Limit CPU usage).

---

## 6. DEVELOPMENT ROADMAP (IMPLEMENTATION ORDER)

1. **Skeleton**: Main loop reading stdin/stdout.
2. **Schema**: Pydantic models matching Rust protocol.
3. **Watchdog**: Memory monitor thread.
4. **Integration**: Docling logic (The heaviest part).

---

## 7. WINDOWS 11 CONSIDERATIONS

Since we're running on **Windows 11 Home**:

1. **Signal Handling**: Use `signal.SIGINT` and `signal.SIGBREAK` (Windows-specific).
2. **Path Handling**: Always use `pathlib.Path` for cross-platform compatibility.
3. **Process Management**: Use `subprocess.CREATE_NO_WINDOW` flag when spawning.
4. **Memory Monitoring**: `psutil` works on Windows, but some metrics differ.

---

**VERSION**: 1.0.0  
**STATUS**: Approved  
**OWNER**: TachFileTo Core Team
