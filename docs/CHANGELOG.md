# TachFileTo Changelog

All notable changes to this project will be documented in this file.
This project adheres to **Semantic Versioning**.

---

## [1.0.0-core+] - 2026-02-26
### ✨ Product Realignment (Phase 10)
- **Identity Pivot**: Transitioned from "Forensic Workstation" to "Offline Document Processor".
- **Documentation Overhaul**: Introduced the "Trinity of Truth" (PRODUCT_SPEC, SYSTEM_ARCHITECTURE, DEVELOPMENT_GUIDE).
- **Architecture Freeze**: Canonical Rust `iron_engine` established as the single source of truth.
- **Tech Stack Consolidation**: Standardized on Svelte 5, Tauri 2.0, and Pure Rust.

### 🚀 Engine Stabilization
- **Streaming Ingestor**: O(1) memory usage during ingestion.
- **Immutable AST**: Byte-deterministic structure representation.
- **O(n) Diff**: High-efficiency comparison engine with StableId.
- **API Lock**: Formalized Tauri IPC command set.

### 🧹 Cleanup
- Archived 15 legacy forensic/compliance specifications.
- Removed obsolete Python/Docling bridge references.
- Purged legacy build artifacts and system junk files.
