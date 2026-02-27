# TachFileTo

**Offline Large Document Processor for AI & Version Comparison**

> Process massive PDF/DOCX files. Clean them for AI. Compare versions with precision. No cloud. No lag.

---

## The Problem

Engineers, accountants, and lawyers work with documents that standard tools cannot handle:

- Files **over 50MB** that crash viewers and editors.
- **Scanned documents** that cannot be searched or read by AI.
- Raw copy-paste into AI produces **garbage** — broken tables, corrupt headers.
- Comparing two contract versions with traditional diff tools gives **cascade false positives**.

TachFileTo solves all four. Entirely offline.

---

## What It Does

### Mode 1 — Clean (Document Ingestion)

Converts large PDF or DOCX files into clean, structured output:

- **Auto-OCR**: Detects whether a document has a text layer; applies OCR automatically if not.
- **Streaming processing**: Handles files page-by-page. Memory usage does not grow with file size.
- **Three export formats**: AI-ready Markdown, structured DOCX, searchable PDF.
- **One-click Copy for AI**: Optimized prompt-ready output.

### Mode 2 — Compare (Version Delta)

Compares two versions of a document with deterministic, structural precision:

- **StableId anchoring**: Table rows and headings are identity-anchored. A row inserted in the middle does not trigger cascade false alarms.
- **Numerical Drift detection**: Reports value changes ≥ configurable epsilon (default: 1.0).
- **DiffReport**: Exportable summary in JSON or PDF.
- **O(n) performance**: Comparison time scales linearly, not exponentially.

---

## Technology

| Layer     | Technology                                    |
|-----------|-----------------------------------------------|
| Engine    | Pure Rust (Edition 2024), `iron_engine` crate |
| Shell     | Tauri 2.0 (Windows / macOS)                   |
| Interface | Svelte 5 + Tailwind v4                        |
| OCR       | Tesseract (offline, Vietnamese-optimized)     |

---

## Canonical Documentation

| Document | Role |
|---|---|
| [PRODUCT_SPEC.md](docs/PRODUCT_SPEC.md) | What V1.0 builds and what it does not |
| [SYSTEM_ARCHITECTURE.md](docs/SYSTEM_ARCHITECTURE.md) | Technical contracts and invariants |
| [DEVELOPMENT_GUIDE.md](docs/DEVELOPMENT_GUIDE.md) | How to build and contribute |

---

## What It Is Not

- Not a cloud service — runs entirely offline
- Not a document editor — output is structured data, not a formatted file
- Not a legal compliance engine — no audit trails, no regulatory alignment
- Not an AI model — it prepares documents for AI, it is not AI itself

---

## Status

**Core Engine**: `v1.0.0-core+` ✅ — tagged, tested, API-locked.
**UI**: In progress. Not yet production-ready. CLI/engine testing only.
**Phase**: Formalizing V1.0 Identity.

---

*Built for professionals working with large technical documents. No fluff. No cloud dependency.*
