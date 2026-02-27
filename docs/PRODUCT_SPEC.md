# PRODUCT_SPEC.md — TachFileTo V1.0

**Version**: 1.0.0  
**Status**: Canonical. Locked.  
**Last updated**: 2026-02-26

> This document is the feature contract for TachFileTo V1.0.
> It defines exactly what the product is, what it does, and what it will never do.
> Any feature not mentioned here does not exist in V1.0.

---

## 1. Identity

**TachFileTo** is an **Offline Large Document Processor**.

Its two core jobs:
1. Convert massive, messy documents into clean, AI-ready structured output (**Clean Mode**).
2. Compare two versions of a document with structural precision, not text diffing (**Compare Mode**).

It is a **utility tool**, not a compliance system.  
It is a **precision instrument**, not a general-purpose document editor.  
It is **entirely offline**, with zero network calls during any processing operation.

---

## 2. Target User

| User Type | Pain Point |
|---|---|
| BOQ / Proposal Engineers | Hundreds of pages of bid documents, need clean tables for AI review |
| Accountants & Legal Staff | Long contracts (100–500 pages), need to track value changes between versions |
| AI Power Users | Need to feed large, structured document context into LLMs without prompt noise |

---

## 3. Core Problems Solved

1. **Size Lag**: Files >50MB crash or lag standard PDF/Word viewers. TachFileTo processes them in <60 seconds.
2. **Scan Blindness**: Scanned PDFs have no text layer. Auto-OCR makes them searchable and AI-readable.
3. **Format Noise**: Copy-pasting large documents into AI produces broken tables and junk headers. Clean Mode eliminates this.
4. **Comparison Hell**: Traditional `diff` tools on documents produce cascade false positives when a row shifts. TachFileTo uses `StableId` anchoring to prevent this.

---

## 4. Features — Mode 1: CLEAN

**Purpose**: Ingest a large document and produce clean, structured output.

| Feature | Detail |
|---|---|
| Supported inputs | PDF (text layer or scanned), DOCX |
| Max tested file size | 100MB+ |
| Auto-OCR | Triggered automatically if no text layer is detected in PDF |
| Export: Markdown | Clean `.md` with correct heading hierarchy, ready for LLM context |
| Export: DOCX | Reconstructed structured document |
| Export: Searchable PDF | Original layout with embedded text layer |
| Copy for AI | One-click export of optimized AI prompt context |
| Memory model | Sawtooth RAM — processes page-by-page, drops after write |
| UI feedback | Real-time streaming progress: page X of Y |

---

## 5. Features — Mode 2: COMPARE

**Purpose**: Compare two versions of a document and report structural and numerical changes.

| Feature | Detail |
|---|---|
| Supported inputs | Two files (A and B): PDF or DOCX |
| StableId anchoring | Each heading and table row has a stable identity hash. Row insertions do not cause cascades. |
| Numerical Drift | Detects value changes ≥ epsilon (default: 1.0 VND) |
| Structural changes | Reports section add/remove, table row add/remove |
| DiffReport export | JSON and PDF format |
| Processing model | Parallel streaming of both files simultaneously |
| O(n) guarantee | Comparison time is linear in document size |

---

## 6. Performance Guarantees

These are hard contracts, not aspirations:

| Metric | Guarantee |
|---|---|
| Processing speed | 100MB file in <60 seconds |
| Memory behavior | Sawtooth pattern (O(1) memory slope) — never linear growth |
| UI responsiveness | Frontend never freezes during backend processing |
| Rendering | 100,000+ data rows at 60fps via virtualized list |
| Offline | Zero network calls at any point during processing |
| Determinism | Same file + same version = byte-identical Markdown output |

---

## 7. Non-Goals — What V1.0 Does NOT Do

The following are **explicitly out of scope** for V1.0. They do not exist. They are not planned.

| Excluded Feature | Reason |
|---|---|
| SHA-256 / Ed25519 chain of custody | We are a utility, not a forensic audit system |
| Legal compliance alignment (Nghị định 254, etc.) | Out of scope by design |
| Verdict rendering ("This contract differs by X%") | Interpretation is the user's job, not ours |
| Immutable ledger of corrections | No compliance requirement in scope |
| Cloud processing / remote API | Offline is a core invariant, not a feature toggle |
| Python IPC or scripting runtime | No Python in the stack |
| Bloomberg-style dashboards | Not a BI tool |
| AI model integration (calling LLM APIs) | We prepare data for AI. We are not AI. |
| Style preservation (fonts, colors, layout) | We prioritize data fidelity over visual fidelity |

---

## 8. IPC Data Contracts

These are the exact shapes of data crossing the Tauri IPC boundary. Both Rust and Svelte must conform.

### `DocumentSummary`

Returned by `process_document`. Opaque to the UI — it is passed back to the engine for compare operations.

```ts
{
  id: string;           // UUID (string, not number — avoids JS float precision loss)
  source_path: string;  // Absolute path of the original file
  total_pages: number;  // u32
  has_ocr: boolean;     // true if OCR was applied to any page
}
```

### `DiffReport`

Returned by `compare_documents`.

```ts
{
  is_identical: boolean;   // true if zero deltas
  total_deltas: number;    // u32
  deltas: Delta[];
}
```

### `Delta`

```ts
{
  node_id: string;         // StableId of the affected heading or table row
  kind: "Added" | "Removed" | "Modified";
  location: string;        // Human-readable: e.g. "Bảng 3 > Hàng 12"
  old_value?: string;      // Present only for "Modified"
  new_value?: string;      // Present only for "Modified"
}
```

### `ProcessError` (Error Codes)

Backend returns enum codes. **Never raw error strings.** Frontend maps to Vietnamese messages.

```rust
enum ProcessError {
    FileTooLarge,      // File vượt ngưỡng 500MB
    OcrFailed,         // OCR engine không phản hồi
    UnsupportedFormat, // Định dạng file không được hỗ trợ
    UserCancelled,     // Người dùng hủy tiến trình
    IoError,           // Lỗi đọc/ghi file
    EnginePanic,       // Lỗi nội bộ không xác định
}
```

---

## 9. Failure Behavior

Rules for how the system behaves when things go wrong. These are constraints, not suggestions.

| Scenario | Behavior |
|---|---|
| OCR fails on a single page | `Soft-skip`: continue processing remaining pages. Mark failed page as `[OCR_FAILED_PAGE_N]` in Markdown output. Do not abort. |
| OCR binary not found in PATH | Abort immediately. Return `ProcessError::OcrFailed`. Do not attempt processing without OCR when a scan is detected. |
| File > 500MB | Reject before loading. Return `ProcessError::FileTooLarge`. Do not begin ingest. |
| Unsupported file format | Reject before loading. Return `ProcessError::UnsupportedFormat`. |
| User cancels mid-process | Abort the engine worker. Release all memory. Delete any temp files written to disk. Complete within 500ms. Return `ProcessError::UserCancelled`. |
| File A succeeds, File B fails in Compare | Abort compare entirely. Return error for File B. Do not show a partial diff. |
| Structural mismatch (A and B have zero common `StableId`) | Complete normally. Return `DiffReport` with all A nodes as `Removed` and all B nodes as `Added`. Do not special-case this. |
| Epsilon-only diff (all changes < 1.0) | Return `DiffReport` with `is_identical: true` and `total_deltas: 0`. Epsilon filtering is applied before reporting. |
| RAM exceeds 2GB during processing | Abort. Return `ProcessError::EnginePanic`. Log to local file. |

---

## 10. Language Policy (vi-VN)

The entire user-facing interface is Vietnamese. This is a hard constraint, not a preference.

### Rule 1 — Display language

All labels, buttons, toasts, progress messages, and error messages must be in Vietnamese.

### Rule 2 — Technical nouns are exempt

The following terms are **not translated**. They are universal technical nouns:

`PDF`, `DOCX`, `Markdown`, `OCR`, `AI`, `RAM`, `MB`, `GB`

### Rule 3 — Standard glossary

All developers must use these mappings consistently. No synonyms allowed.

| English | Vietnamese (canonical) |
|---|---|
| Clean / Process | Xử lý |
| Compare | So sánh |
| Export | Xuất file |
| Processing… | Đang xử lý… |
| Completed | Hoàn tất |
| Failed | Thất bại |
| Cancel | Hủy |
| Loading / Ingesting | Đang nạp dữ liệu |
| Numerical difference | Sai lệch số liệu |
| Structural change | Thay đổi cấu trúc |

### Rule 4 — Error message architecture

Backend returns only `ProcessError` enum codes. Frontend owns all human-readable strings.

```ts
// messages.vi.ts — single source of truth for all UI strings
export const ERROR_MESSAGES: Record<ProcessError, string> = {
  FileTooLarge:      "Tệp quá lớn. Vui lòng chọn tệp dưới 500MB.",
  OcrFailed:         "Không thể nhận dạng văn bản. Kiểm tra lại cài đặt OCR.",
  UnsupportedFormat: "Định dạng tệp không được hỗ trợ.",
  UserCancelled:     "Đã hủy. Dữ liệu tạm thời đã được xóa.",
  IoError:           "Lỗi đọc tệp. Kiểm tra quyền truy cập thư mục.",
  EnginePanic:       "Lỗi hệ thống không xác định. Vui lòng thử lại.",
};
```

No component may hardcode a user-facing string directly.

---

**End of PRODUCT_SPEC.md — TachFileTo V1.0**
