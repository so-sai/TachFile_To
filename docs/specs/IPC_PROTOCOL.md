🚨 **TÀI LIỆU ĐÃ LỖI THỜI - KHÔNG SỬ DỤNG**
**Version thực tế:** V2.3 (Perception Engine + Polars 0.52)
**Cập nhật cuối:** 2025-12-26
**Trạng thái:** ARCHIVED - Chỉ để tham khảo lịch sử
→ Xem [ARCHITECTURE_V2.3.md](file:///e:/DEV/TachFile_To/docs/specs/ARCHITECTURE_V2.3.md) để biết source of truth

# IPC PROTOCOL v1.1 0 SPECIFICATION (Zero-copy Arrow)

**Version:** 2.0.0 (2025 Standard)  
**Status:** Design Specification (Pivoted from v1.1)  
**Last Updated:** 2025-12-25

---

## 🎯 Overview (The 2025 Pivot)

Based on the **Veritas Atomic Simulator** audit, TachFileTo has migrated from **Stdio JSON (Legacy)** to **Shared Memory Apache Arrow (v2.0)** to eliminate serialization bottlenecks in a **No-GIL Python 3.14** environment.

### Protocol v2.0 Principles

1.  **Zero-copy Data Transfer**: Large payloads (images, tables) are written directly to memory-mapped segments.
2.  **Schema-First Serialization**: Data is serialized using Apache Arrow's binary IPC format.
3.  **Hybrid Transport**:
    - **Stdio**: Small JSON envelopes for metadata and control signals.
    - **Shared Memory (mmap)**: High-throughput binary data.

---

## 1. HYBRID MESSAGE STRUCTURE

### Control Envelope (Stdio JSON)

```json
{
  "protocol_v": "2.0.0",
  "msg_id": "uuid-v4",
  "type": "CMD_EXTRACT_EVIDENCE",
  "payload": {
    "metadata": { "file_path": "...", "page": 5 },
    "shm_handle": {
       "segment_id": "tachfile_shm_550e",
       "offset": 0,
       "size": 1048576,
       "format": "arrow_stream"
    }
  }
}
```

### Data Payload (Shared Memory / Arrow)

Evidence items and Tables are transferred as **Arrow RecordBatches** within the Mapped Segment.

| Data Type | Arrow Schema | Rationale |
| :--- | :--- | :--- |
| **Evidence** | `[data: binary, width: i32, height: i32, format: string]` | Raw bitmap transfer, avoids Base64 |
| **Ledger** | `[stt: i32, code: string, name: string, quantity: f64, ...]` | High-speed processing of 100k+ rows |

---

## 2. SHARED MEMORY LIFECYCLE

### 2.1 Allocation (Rust Core)
Rust Core initializes a `memmap2` segment (default 100MB) on startup.
- **Windows**: `Global\TachFileTo_IPC` (Shared file mapping)
- **Unix**: `/dev/shm/tachfileto_ipc`

### 2.2 Writing (Python Worker)
In Python 3.14, workers use `pyarrow.memory_map` to write RecordBatches directly into the segment.
```python
with pa.memory_map(shm_path, 'r+') as mmap:
    # Write directly into fixed buffer
    writer = pa.ipc.new_stream(mmap, schema)
    writer.write_batch(batch)
```

---

## 3. UPDATED MESSAGE TYPES

| Type | Tier | Purpose |
| :--- | :--- | :--- |
| `CMD_STREAM_START` | Stdio | Negotiate SHM segment size and handle |
| `CMD_PROCESS_PDF` | Stdio | Trigger parallel processing in sub-interpreters |
| `RES_DATA_READY` | Stdio | Signal that data is written to SHM and ready for Rust |
| `RES_ERROR_V2` | Stdio | Hierarchical errors including SHM corruption |

---

## 4. PERFORMANCE TARGETS (2025 ERA)

| Metric | Target (v2.0) | vs v1.1 (JSON) |
| :--- | :--- | :--- |
| **10k Rows Latency** | **<10ms** | 50x Faster |
| **Image Extraction** | **<2ms** | 100x Faster |
| **CPU Usage (IPC)** | **<1%** | 20x Lower |
| **Memory Overhd** | **~0MB (Zero-copy)** | Infinite Reduction |

---

## 5. REPOSITORY IMPACTS

### 5.1 Rust Core ([`crates/tachfileto-core`](file:///e:/DEV/TachFile_To/crates/tachfileto-core))
- Added `arrow` and `memmap2` to `Cargo.toml`.
- New Module: `ipc/shm_manager.rs`.

### 5.2 Python Worker ([`py-worker`](file:///e:/DEV/TachFile_To/backend))
- Updated `requirements.txt` with `pyarrow>=16.0.0`.
- Transitioning to **Sub-interpreter** isolation model.

---

**Detailed Transition Plan:**
- [ARCHITECTURE_MASTER_V1.1.md](file:///e:/DEV/TachFile_To/docs/specs/ARCHITECTURE_MASTER_V1.1.md) (Updated)
- [IMPLEMENTATION_PLAN.md](file:///C:/Users/Admin/.gemini/antigravity/brain/5e5ab505-e7ba-4641-8ef1-7ef60aa55493/implementation_plan.md) (Updated)
