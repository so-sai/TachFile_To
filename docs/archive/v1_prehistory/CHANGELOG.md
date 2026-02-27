# TachFileTo Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.4.0] - 2025-12-26

### 🚀 Major Architecture Overhaul

**Pure Rust Excel Engine** - Eliminated Python dependency for Excel processing, achieving 6.8x performance improvement.

### Added
- **Calamine 0.32 Integration**: Direct Excel file parsing in Rust
  - Support for .xlsx, .xlsb, .xls formats
  - Full Excel 2025 compatibility
  - OADate and ISO DateTime handling
  - ~200,000 rows/second processing speed

- **Terminology Normalizer V1.0**: Automatic column name standardization
  - Fuzzy matching for Vietnamese QS terminology
  - 8 standard column mappings (ma_hieu, dien_giai, dvt, khoi_luong, don_gia, thanh_tien, trang_thai, ngay_tao)
  - Case-insensitive substring matching
  - Handles both Vietnamese (with/without diacritics) and English column names

- **Performance Benchmarks**: Documented 6.8x average speedup over V2.2

### Changed
- **Data Flow Architecture**: Excel → Calamine (Rust) → Polars → UI (was: Excel → Python → Rust → UI)
- **Polars Version**: Upgraded to 0.52 with optimized feature set
- **Memory Management**: 30% reduction in RAM usage via Calamine's optimized buffers

### Fixed
- **Namespace Conflict**: Resolved `DataType` collision between Polars and Calamine by using `calamine::Data` enum
- **Build Errors**: Fixed Polars 0.52 feature configuration (removed non-existent `calamine` feature)
- **Edition Warning**: Updated Cargo.toml to use valid Rust edition "2021"

### Performance
- 10k rows: 0.8s → 0.12s (6.7x faster)
- 100k rows: 8.5s → 1.2s (7.1x faster)  
- 1M rows: 95s → 14s (6.8x faster)
- Memory: 850MB → 600MB (29% reduction)

### Documentation
- Created `ARCHITECTURE_V2.4.md` with complete technical specification
- Added maintenance guide for Calamine and Polars upgrades
- Documented namespace conflict resolution pattern

---

## [2.3.0] - 2025-12-24

### Added
- **IPC Protocol v1.0**: Rust-Python communication protocol
  - Message envelope with UUID tracking
  - Handshake, Success, Error, Progress message types
  - NDJSON over stdio pipes

- **IPC Router**: Async message routing with tokio
  - Pending request management via HashMap
  - Oneshot channels for response delivery
  - Unit tests for success/error flows

- **IPC Manager**: Python subprocess management
  - Process spawning with stdio pipes
  - Absolute path resolution for Windows compatibility
  - Graceful shutdown handling

- **Python Worker**: Protocol-compliant event loop
  - Pydantic v2 message validation
  - Mock Docling engine integration
  - Stderr logging (stdout reserved for IPC)

### Verified
- End-to-end IPC handshake: Rust ↔ Python ↔ Rust
- Integration test binary (`examples/ipc_test.rs`)

---

## [2.2.0] - 2025-12-20

### Added
- **Legacy Font Converter**: Vietnamese TCVN3/VNI to Unicode
  - 2-pass heuristic: frequency analysis + dictionary validation
  - Comprehensive character mapping tables
  - Unit tests for TCVN3 conversion

- **Rust Workspace Structure**: `crates/tachfileto-core`
  - Text processing module (`text/legacy_fonts.rs`)
  - IPC module (`ipc/protocol.rs`, `ipc/router.rs`)

### Dependencies
- Added: `lazy_static`, `uuid`, `chrono`, `serde`, `serde_json`, `tokio`

---

## [2.1.0] - 2025-12-15

### Added
- Initial Tauri 2.0 setup
- React 19 frontend scaffold
- Basic project structure

---

## [2.0.0] - 2025-12-10

### Added
- Project inception
- Core concept: High-performance Excel processing for Vietnamese QS industry

---

[2.4.0]: https://github.com/so-sai/TachFile_To/compare/v2.3.0...v2.4.0
[2.3.0]: https://github.com/so-sai/TachFile_To/compare/v2.2.0...v2.3.0
[2.2.0]: https://github.com/so-sai/TachFile_To/compare/v2.1.0...v2.2.0
[2.1.0]: https://github.com/so-sai/TachFile_To/compare/v2.0.0...v2.1.0
[2.0.0]: https://github.com/so-sai/TachFile_To/releases/tag/v2.0.0
