---
description: Standard procedure for creating robust Rust crates (Elite Standard 2026)
---

# Elite Crate Fabrication Protocol

> **Purpose:** To create specialized, high-performance, and panic-free Rust crates.
> **Standard:** Enforces `elite-rust` skill, TDD Layers, and Contract-First design.

## 1. Scaffold & Constitution

1. **Initialize Crate:**
   ```bash
   cargo new --lib libs/<crate_name>
   ```
2. **Define Dependencies (Pinned):**
   - Edit `Cargo.toml`.
   - **CRITICAL:** Pin Polars to `=0.52.0` (or latest approved).
   - Use `thiserror` for library errors.
   - Use `serde` with `derive`.

## 2. Contract Definition (Iron Law)

1. **Define Structs First:**
   - Create input/output structs in `iron_table` (or internal `contract.rs` if isolated).
   - Enforce `#[serde(rename_all = "camelCase")]` for JSON/TS interop.
   - Define strict types (Enum vs String).

## 3. TDD Layer 1: Structural Determinism

> **Rule:** No core logic code until these tests fail.

1. **Create Test Skeleton:**
   - `test_exact_schema_match`: Verify strict 1:1 mapping.
   - `test_rejection_rules`: Verify invalid inputs return `Err`.
   - `test_determinism`: Verify `f(x) == f(x)` byte-for-byte.
2. **Implement Transformer:**
   - Use explicit `Series` construction (No inference).
   - Use `.into_column()` for Polars 0.52+.
   - Add explicit validation (e.g., row count checks).

## 4. TDD Layer 2: Arithmetic Truth

> **Rule:** Thresholds and Math must be exact.

1. **Create Arithmetic Tests:**
   - `test_threshold_boundaries`: Test `4.99` vs `5.00`.
   - `test_exact_aggregation`: assert_eq! with NO EPSILON for financials.
   - `test_idempotence`: End-to-end result stability.
2. **Implement Calculator:**
   - Implement logic using Polars expressions or iterators.
   - Handle `NaN` / `Inf` explicitely (`unwrap_or`).
   - Eliminate all `unwrap/expect`.

## 5. Release Audit (The Gate)

1. **Determinism Scan:**
   - Grep for `hash`, `random`, `sample`, `iter`.
   - **BAN:** `Utc::now()` inside logic (Pass as argument).
2. **Panic Audit:**
   - Grep for `unwrap()`, `expect()`.
   - **Allowed:** Tests only.
   - **Forbidden:** Logic paths.
3. **Performance Baseline:**
   - Logic < 50ms (or budget limits).
4. **Freeze:**
   - Mark Mission/Crate as **FROZEN**.

---

**Usage:**
Run this protocol for EVERY new crate (`iron_core`, `iron_bridge`, etc.).
