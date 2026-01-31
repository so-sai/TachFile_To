# ADR-002: Pivot to Apache Arrow IPC (Zero-Copy)

| Status | Accepted |
|:---:|:---:|
| **Date** | 2025-12 (Phase 3.5) |
| **Authors** | Veritas Atomic Simulator |
| **Context** | Performance Bottleneck in Legacy JSON IPC |

## Context and Problem Statement
In Phase 3, the application faced severe performance degradation when transferring large datasets (100k+ rows) between the Python Worker and the Rust/Tauri Core.
- **Legacy Protocol**: Stdio Pipe with JSON serialization + Base64 encoding for binary data.
- **Metrics**: 4.8 seconds latency for 100k rows, 60% CPU usage.
- **Root Cause**: Excessive memory copying and serialization overhead (Python Heap -> JSON String -> Bytes -> Pipe -> Rust String -> JSON Parse -> Rust Struct).

## Decision Drivers
1. **Performance Requirement**: Sub-100ms latency for 100k rows to achieve "Institute" grade UX.
2. **Python 3.14 Capability**: The new No-GIL build allows true parallelism, which JSON serialization blocked.
3. **Memory Constraint**: Avoiding memory spikes during data transfer.

## Considered Options
1. **Optimize JSON**: Use `orjson` and compression. (Rejected: Still requires serialization/copy).
2. **Named Pipes**: Native OS pipes. (Rejected: Still stream-based, requires serialization).
3. **Apache Arrow + Shared Memory (Selected)**: Map a memory segment readable by both processes.

## Decision Outcome
Chosen option: **Apache Arrow IPC over Memory Mapped Files (MMAP)**.

### Architectural Changes
1. **Hybrid Signaling**:
   - **Control Plane**: Uses TCP Loopback (Ephemeral Ports) on Windows for reliable, non-blocking signaling.
   - **Data Plane**: Uses Shared Memory (MMAP) for bulk data payload (Arrow RecordBatch).
2. **Zero-Copy**: Python writes to RAM; Rust reads the same RAM address. Zero serialization.

### Consequences
* **Positive**:
  * Throughput increased by **113x** (4.8s -> 42ms).
  * CPU usage dropped by **15x**.
  * Zero memory copies for the payload.
* **Negative**:
  * Complexity: Requires strict schema synchronization and unsafe Rust blocks for MMAP.
  * Dependency: Requires `pyarrow` and `arrow-rs` version alignment.

## Verification
- **Stress Test**: 100k rows verified in 42.7ms.
- **Safety**: `Arc<Mutex<>>` wrapper implemented for thread-safe IPC management in Rust.
