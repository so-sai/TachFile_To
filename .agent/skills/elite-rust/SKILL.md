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

## Extraction (Unified Orchestrator)
- **Architecture**: Use the "Unified Extraction Orchestrator" pattern. Orchestrators must be stateless, purely gaging capability and dispatching to lanes.
- **Stability & Isolation**: Heavy FFI tasks (PDF/Office) MUST be isolated in sub-processes via `WorkerManager`.
- **Fate-sharing**: Workers must monitor `stdin`. If parent drops, worker must exit immediately to avoid "ghost processes".
- **Resource Governor**: Limit maximum concurrent workers using a `Semaphore`. Cap based on CPU core count or memory availability.
- **Contract Enforcement**: All lanes must speak the `ExtractionProduct` JSON protocol.

## Interoperability (IPC Protocol)
- **JSON Stream**: Communication between Main (Rust) and Worker (Python) must be conducted via JSON over `stdin/stdout`.
- **CamelCase Alignment**: All IPC payloads MUST be `camelCase`.
- **Timeout Policy**: Every worker task must have a hard timeout to prevent blocking the dispatcher.
- **Zero-Lag IPC**: Round-trip time (RTT) for IPC MUST NOT exceed 20ms. Use persistent worker pools to avoid cold-start overhead. Any refactoring that degrades IPC performance beyond this threshold MUST be rejected.

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
