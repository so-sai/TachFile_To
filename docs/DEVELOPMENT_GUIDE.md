# DEVELOPMENT_GUIDE.md — TachFileTo V1.0

**Version**: 1.0.0  
**Status**: Canonical. Locked.  
**Last updated**: 2026-02-26

> This document defines how to build, run, and contribute to TachFileTo V1.0.
> Read SYSTEM_ARCHITECTURE.md first for the technical context behind these rules.

---

## 1. Prerequisites

| Tool | Requirement | Install |
|---|---|---|
| Rust | Edition 2024, MSRV 1.82+ | `rustup update stable` |
| Node.js | LTS (20+) | [nodejs.org](https://nodejs.org) |
| Tesseract | Any stable, `tesseract` in PATH | System package or [UB Mannheim builds](https://github.com/UB-Mannheim/tesseract/wiki) |
| Tauri CLI | v2.0+ | `cargo install tauri-cli --version "^2"` |

> **Note on Tesseract**: Vietnamese language data (`vie.traineddata`) must be installed. TachFileTo will panic if `tesseract` binary is not found in PATH.

---

## 2. Quick Start

### Step 1 — Install frontend dependencies

```powershell
cd ui
npm install
```

### Step 2 — Run in development mode

```powershell
# From project root
npm run tauri dev
```

This compiles the Rust backend and starts the Svelte 5 dev server with hot-reload.

---

## 3. Running Engine Tests Only

To verify the `iron_engine` core independently (no Tauri, no UI):

```powershell
cd libs/iron_engine
cargo test
```

All integration tests use only the public Facade (`lib.rs`). Do not write tests that import private modules.

---

## 4. Repository Structure

```
/
├── libs/
│   ├── iron_engine/       # Core engine — API-locked at v1.0.0-core+
│   └── iron_table/        # Table heuristics
│
├── src-tauri/             # Tauri shell (bridge between UI and engine)
│
├── ui/                    # Svelte 5 frontend
│   └── src/
│       ├── routes/        # Page components
│       ├── lib/           # Shared utilities
│       └── stores/        # Runes-based state
│
├── docs/                  # Canonical V1.0 spec set
│   ├── PRODUCT_SPEC.md
│   ├── SYSTEM_ARCHITECTURE.md
│   └── DEVELOPMENT_GUIDE.md (this file)
│
├── samples/               # Test PDF/DOCX files
└── README.md
```

---

## 5. Coding Rules

### Rust (Backend / Engine)

- **Zero Panic Policy**: All IPC-facing functions must return `Result<T, ProcessError>`. Use `thiserror` for error types.
- **No `.unwrap()` in production code**: Use `?` propagation or explicit `.expect()` with a clear diagnostic message.
- **Manual drop discipline**: After processing each page/section, explicitly call `drop()` on large buffers. Do not rely on scope exit alone.
- **Lock-free concurrency**: Use `crossbeam` channels or `mpsc::sync_channel` for streaming. Avoid `Mutex<Vec<_>>` for hot paths.
- **No `unsafe` without comment**: Any `unsafe` block must have a `// SAFETY:` justification comment.
- **`cargo fmt` + `cargo clippy`** must pass before any commit. No warnings suppressed silently.

### Svelte 5 (Frontend)

- **Rune-only state**: Use `$state` and `$derived`. The Svelte 4 `writable()` / `readable()` store API is banned.
- **Zero business logic in components**: Components receive data and dispatch events. No calculations.
- **Virtualized lists**: Any rendered list with >50 items must use a virtual scroller.
- **TypeScript**: All Svelte components and `.ts` utility files must be typed. `any` requires a justification comment.

### Language

- **UI text**: Vietnamese only.
- **Code, comments, commit messages**: English only.

---

## 6. Commit Convention

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(engine): add OCR fallback for embedded-image PDFs
fix(diff): correct epsilon boundary for zero-value rows
refactor(ui): migrate CompareView to $state runes
docs: update SYSTEM_ARCHITECTURE with IPC command signatures
```

---

## 7. Internationalization Rules (i18n)

All user-facing text is Vietnamese. This is enforced at the code level, not by convention.

### The `messages.vi.ts` Pattern

Create `ui/src/lib/messages.vi.ts` as the **single source of truth** for all user-visible strings:

```ts
// ui/src/lib/messages.vi.ts
import type { ProcessError } from './types';

export const ERROR_MESSAGES: Record<ProcessError, string> = {
  FileTooLarge:      'Tệp quá lớn. Vui lòng chọn tệp dưới 500MB.',
  OcrFailed:         'Không thể nhận dạng văn bản. Kiểm tra lại cài đặt OCR.',
  UnsupportedFormat: 'Định dạng tệp không được hỗ trợ.',
  UserCancelled:     'Đã hủy. Dữ liệu tạm thời đã được xóa.',
  IoError:           'Lỗi đọc tệp. Kiểm tra quyền truy cập thư mục.',
  EnginePanic:       'Lỗi hệ thống không xác định. Vui lòng thử lại.',
};

export const UI = {
  mode_clean:        'Xử lý',
  mode_compare:      'So sánh',
  action_export:     'Xuất file',
  action_cancel:     'Hủy',
  status_processing: 'Đang xử lý…',
  status_done:       'Hoàn tất',
  status_failed:     'Thất bại',
  status_loading:    'Đang nạp dữ liệu…',
} as const;
```

### Enforcement rules

1. **No component may have a raw string literal** for user-visible text. Always import from `messages.vi.ts`.
2. **No English text in production UI**. `console.log` and code comments may use English.
3. **Technical nouns are not translated**: `PDF`, `DOCX`, `Markdown`, `OCR`, `AI`, `RAM`, `MB`, `GB`.
4. **Backend error strings are forbidden**: Rust returns `ProcessError` enum variants only. The frontend does the lookup.

---

## 8. The Permanent Forbidden List

The following are **permanently banned** from TachFileTo V1.0.  
This is not a preference. This is a hard technical constraint.

| Forbidden | Category | Reason |
|---|---|---|
| Python / PyO3 / subprocess python | Runtime | Adds interpreter dependency, breaks offline model |
| Polars | Data library | Replaced by `iron_engine` native logic |
| Docling | ML document library | Heavy ML dependency, non-deterministic |
| React / Next.js / Vue | UI framework | Stack is locked to Svelte 5 |
| Any cloud SDK | Network | Offline is a core invariant |
| SHA-256 chain / Ed25519 | Audit feature | Out of product scope |
| Raw error strings from Rust | i18n | All errors are `ProcessError` enum codes |
| Hardcoded UI strings in components | i18n | All strings must come from `messages.vi.ts` |

> **If a PR introduces any of the above**, it is rejected without review.

---

## 9. Adding New Features

Before writing any code for a new feature:

1. Check `PRODUCT_SPEC.md` — is the feature in scope for V1.0?
2. Check `SYSTEM_ARCHITECTURE.md` — which layer handles this? If it's computation, it goes in `iron_engine`.
3. If the feature is out of scope → do not implement it for V1.0.

There is no shortcut around this process.

---

**End of DEVELOPMENT_GUIDE.md — TachFileTo V1.0**
