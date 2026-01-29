---
name: elite-rust
description: Rust Elite Standards (Edition 2024, Safe & Robust)
---

# Rust Elite Standards (Edition 2024)

## Architecture
- **Edition**: Must use Rust Edition 2024.
- **Dependencies**: Use Standard Polars `0.52.x` (ensure patch version compatibility).

## Safety & Error Handling
- **No Panic Policy**: Strictly forbid `unwrap()` and `expect()` in business logic.
- **Typed Errors**: Use `thiserror` for library errors and `anyhow` for application-level errors.
- **Async**: Use `tokio` for async runtime unless specified otherwise.

## Testing & Quality (Atomic Simulator VAS)
- **TDD Requirement**: "No Test, No Commit". Every feature must have a "Red Phase" failing test before logic implementation.
- **Performance Benchmarks**: Core operations (e.g., Ledger queries) must meet sub-100ms targets for 1000 rows.
- **Verification**: Every Mission must conclude with a `walkthrough.md` documenting test results.

## Interoperability (Tauri/TS)
- **Data Contract**: All event payloads between Rust and TypeScript MUST use `camelCase`. Use `#[serde(rename_all = "camelCase")]` on structs.

## Observability
- **Traceability**: Every result-bearing function must be traced or logged using the `tracing` crate. Use `#[tracing::instrument]` on core logic.

## Extraction
- **Pipeline**: Use the "Unified Extraction Pipeline" for all PDF, Docx, Xlsx, and MD processing.

## Refactor & Safety Audit (Elite Mandatory)
- **Unsafe Policy**
  - `unsafe` blocks are forbidden by default.
  - If unavoidable:
    - Must be isolated in a single module.
    - Must include SAFETY comments explaining invariants.
    - Must be reviewed and traced.

- **Refactor Discipline**
  - Eliminate all `unwrap()` / `expect()` (including tests & examples).
  - Replace with `Result<T, E>` and `?`.
  - No silent fallback.

- **Public API Contract**
  - Any public function that can fail MUST return `Result`.
  - Error types must be explicit at library boundaries.

- **Refactor Workflow**
  1. Analyze module responsibilities.
  2. Identify unsafe / panic / implicit failure.
  3. Propose refactor plan **before applying major logic changes**.
  4. Refactor module-by-module under `src/`.
  5. Update call sites, tests, examples, and documentation.
