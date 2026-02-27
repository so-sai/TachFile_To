# SYSTEM_ARCHITECTURE.md — TachFileTo V1.0

**Version**: 1.0.0  
**Status**: Canonical. Locked.  
**Last updated**: 2026-02-26

> This document is the technical contract for TachFileTo V1.0.
> It defines the stack, module boundaries, invariants, and the API Facade that must not drift.
> Violations of these invariants require an explicit architectural decision log, not silent changes.

---

## 1. Technology Stack

| Layer | Technology | Notes |
|---|---|---|
| Shell | Tauri 2.0 | Desktop container, IPC bridge |
| Backend Engine | Rust (Edition 2024) | `iron_engine` crate — the single source of truth |
| Frontend | Svelte 5 + Tailwind v4 + Lucide Icons | Pure renderer |
| OCR | Tesseract (binary, offline) | Vietnamese language pack required |
| Table Parser | `iron_table` (internal crate) | No external parser dependency |
| State Management | Svelte 5 Runes (`$state`, `$derived`) | No Svelte stores |

---

## 2. Repository Structure

```
/
├── libs/
│   ├── iron_engine/       # Core engine (v1.0.0-core+, API-locked)
│   │   ├── src/
│   │   │   ├── lib.rs        # Public API Facade (only entry point)
│   │   │   ├── ast/          # Immutable AST nodes — PRIVATE
│   │   │   ├── diff/         # O(n) diff engine — PRIVATE
│   │   │   ├── ingestor/     # Streaming file reader — PRIVATE
│   │   │   └── exporter/     # Markdown/DOCX/PDF writers — PRIVATE
│   │   └── tests/            # Integration tests via Facade only
│   └── iron_table/        # Table heuristics crate
│
├── src-tauri/             # Tauri shell (planned)
│   └── src/
│       ├── main.rs           # Entry point, panic guard
│       └── commands.rs       # IPC handlers → calls iron_engine Facade
│
├── ui/                    # Svelte 5 frontend
│   └── src/
│       ├── routes/           # Page-level components
│       ├── lib/              # Shared utilities
│       └── stores/           # Runes-based state (no writable stores)
│
└── docs/                  # Canonical V1.0 spec set (this directory)
```

---

## 3. The `iron_engine` Core (v1.0.0-core+)

The core is **API-locked**. Internal modules are private. External code interacts **only** through `lib.rs`.

### 3.1 Streaming Ingestor (Layer 1)

- Reads files page-by-page using a bounded `mpsc::sync_channel`.
- **Backpressure invariant**: Producer blocks when the consumer (AST sink) has not yet processed the previous page.
- **Write-through-and-release**: After each section is written (to buffer or disk), it is immediately dropped from memory.
- Memory model: **Sawtooth RAM** — peak per page, not cumulative.

### 3.2 Immutable AST (Layer 2)

- Every structural node (Heading, Table, Row, Cell) is assigned a `StableId`.
- `StableId = fxhash(path + normalized_content)` — survives physical page re-ordering.
- `NumericIndexEntry`: A lightweight 40-byte value type. No heap allocations, no `String` fields.

### 3.3 Deterministic Diff Engine (Layer 3)

- Operates on two `DocumentSummary` objects. Never on raw files.
- `RowStableId = fxhash(normalized_cells)` — row identity is content-based, not position-based.
- Diff is computed using `HashMap`/`HashSet` set-subtraction. **O(n) guaranteed**.
- **Epsilon invariant**: Numerical changes are only reported if `abs(old - new) >= 1.0`.

### 3.4 Public API Facade (`lib.rs`)

The only interface Tauri commands may call:

```rust
/// Process a single document. Returns an opaque, serializable summary.
pub fn process_document(path: &Path) -> Result<DocumentSummary, ProcessError>;

/// Compare two document summaries. Returns a structured diff report.
pub fn compare_documents(a: &DocumentSummary, b: &DocumentSummary) -> DiffReport;
```

All internal types (`AstNode`, `DiffEntry`, `IngestorState`, etc.) are private to the crate.

### 3.5 Canonical Struct Shapes

These are the exact types serialized across the IPC boundary. **Any change here is a breaking change.**

```rust
/// Opaque summary returned after processing a document.
/// The `id` field is a UUID serialized as String (not u64) to prevent
/// JavaScript float precision loss.
#[derive(Serialize, Deserialize)]
pub struct DocumentSummary {
    pub id: String,           // UUID v4
    pub source_path: String,  // Absolute path of original file
    pub total_pages: u32,
    pub has_ocr: bool,        // true if OCR was applied to any page
}

#[derive(Serialize, Deserialize)]
pub struct DiffReport {
    pub is_identical: bool,
    pub total_deltas: u32,
    pub deltas: Vec<Delta>,
}

#[derive(Serialize, Deserialize)]
pub struct Delta {
    pub node_id: String,       // StableId of affected node
    pub kind: DeltaKind,
    pub location: String,      // e.g. "Bảng 3 > Hàng 12"
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub enum DeltaKind { Added, Removed, Modified }

/// Error codes returned by the engine. Never raw strings.
/// Frontend is responsible for mapping these to Vietnamese messages.
#[derive(Serialize, Deserialize, thiserror::Error, Debug)]
pub enum ProcessError {
    #[error("FileTooLarge")]  FileTooLarge,
    #[error("OcrFailed")]     OcrFailed,
    #[error("UnsupportedFormat")] UnsupportedFormat,
    #[error("UserCancelled")] UserCancelled,
    #[error("IoError")]       IoError,
    #[error("EnginePanic")]   EnginePanic,
}
```

---

## 4. Tauri IPC Commands

Tauri commands in `src-tauri/src/commands.rs` are thin adapters. They do no computation.

```rust
#[tauri::command]
async fn process_document(path: String) -> Result<DocumentSummary, ProcessError>;

#[tauri::command]
async fn export_markdown(id: String) -> Result<String, ProcessError>;

#[tauri::command]
async fn export_docx(id: String) -> Result<String, ProcessError>;

#[tauri::command]
async fn export_pdf(id: String) -> Result<String, ProcessError>;

#[tauri::command]
async fn compare_documents(id_a: String, id_b: String) -> Result<DiffReport, ProcessError>;
```

**Rules**:
- IPC commands must not contain business logic. If logic is needed, it belongs in `iron_engine`.
- Error type is always `ProcessError` — never `String`. Frontend maps code → Vietnamese message.

---

## 5. Error Architecture

The error flow is strictly one-directional:

```
iron_engine  →  ProcessError (enum, Serialized)  →  Tauri IPC  →  Frontend
                                                                        ↓
                                                              messages.vi.ts
                                                                        ↓
                                                           Vietnamese UI message
```

**Invariant**: No Rust code may produce a user-visible error string in any language. The engine is language-agnostic. The frontend (`messages.vi.ts`) owns all user-facing copy.

---

## 6. Frontend Invariants (Svelte 5)

1. **Zero calculation**: The frontend renders what Rust returns. It does not compute sums, counts, or diffs.
2. **Rune-only state**: Use `$state` and `$derived` exclusively. `writable()` stores from Svelte 4 are banned.
3. **Virtualized rendering**: Any list or table with >50 rows must use virtual scrolling. DOM must never hold 100k+ nodes.
4. **Streaming progress**: OCR and ingestion progress is reported as a reactive `$state`: `{ current: number, total: number, phase: string }`.

---

## 7. Architectural Invariants (Non-Negotiable)

| Invariant | Rule |
|---|---|
| Offline Absolute | Zero network calls during any processing. No CDN, no telemetry, no cloud fallback. |
| Sawtooth Memory | RAM must not grow linearly with file size. Peak = one page in memory at a time. |
| Deterministic Output | Same input file + same engine version = byte-identical Markdown output. |
| StableId Anchoring | Table rows and headings have content-based identity. Position is irrelevant. |
| No Python | PyO3, Python IPC, or any Python subprocess is permanently forbidden. |
| No Polars | Polars is permanently forbidden. `iron_engine` handles all data operations. |
| Facade Lock | No code outside `iron_engine` may import internal modules. Only `lib.rs` exports. |
| UI = Pure Renderer | Frontend logic is presentation-only. All computation is Rust. |

---

**End of SYSTEM_ARCHITECTURE.md — TachFileTo V1.0**
