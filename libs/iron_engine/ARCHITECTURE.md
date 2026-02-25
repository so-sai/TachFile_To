# TachFileTo Core+ Architecture (v1.0.0-core+)

This document defines the architectural invariants and "Frozen" contracts of the `iron_engine` (Core+) module. **These contracts must not be violated by future development.**

## 1. The Core Philosophy
TachFileTo is a **Deterministic Execution Engine**, not a fuzzy AI parser.
- **No Schema Inference:** Schemas are provided by `validation_engine`.
- **No Fuzzy Matching:** Diffs strictly use `StableId` and `epsilon` boundaries.
- **Pristine State:** The engine must remain memory-efficient regardless of document size.

## 2. The Three Layers of Core+

### Layer 1: Streaming Ingestor (The Circulatory System)
- **Invariant:** Memory footprint must not grow linearly with document size (O(1) memory slope).
- **Mechanism:** `StreamingIngestor` uses a bounded `mpsc::sync_channel` to enforce **Backpressure** on fast producers (e.g., PDF extractors).
- **Write-Through & Release:** The `AstSink` writes nodes directly to disk (or a buffer) and immediately drops them from memory via `finalize_section`.

### Layer 2: Immutable AST (The Skeleton)
- **Invariant:** AST is structurally immutable and deterministic.
- **StableId:** Every Node receives a 64-bit `StableId` derived from `hash(path + content)`. This anchor survives physical page-shifting.
- **NumericIndexEntry:** A lightweight 40-byte valuetype (no heap allocations, no `String`, `Rc`, or `Box`). It tracks purely `(section_id, table_id, row_hash, col_idx) -> f64`.

### Layer 3: Deterministic Diff Engine (The Judicial Branch)
- **Invariant:** Differences are computed structurally, not via plain-text diffs.
- **RowStableId:** Table rows are identity-anchored by the hash of their content (`RowStableId = fxhash(normalized_cells)`), preventing false cascade alarms when a row is inserted in the middle of a table.
- **O(n) Performance:** Diffs are computed using `HashMap` and `HashSet` set-subtractions and joins.
- **Epsilon Tolerance:** Value mismatches are only reported if `abs(old - new) >= 1.0` VND.

## 3. The Public API Lock (The Facade)
The internal modules (`ast`, `diff`, `ingestor`, etc.) are explicitly **private**.
Tauri (or any UI layer) interacts **exclusively** through the exposed Facade in `lib.rs`:

1. `DocumentSummary`: An opaque, JSON-serializable struct containing the extracted indexes.
2. `compare_documents()`: The entry point that accepts two `DocumentSummary` objects and returns a structured `DiffReport`.

## 4. Why API Lock?
To protect the engine's integrity. By locking the AST and internals behind `lib.rs`, the engine is guaranteed to remain deterministic. Any feature drift originating from the UI must be handled by the UI or mapping layer, keeping the core "Pristine."
