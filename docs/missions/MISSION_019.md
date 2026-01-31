# MISSION 019: The Docling Bridge

> **Status:** PLANNING
> **Owner:** Iron Engine Team
> **Date:** 2026-01-31

## 1. OBJECTIVE

Connect the "Pure" Iron Engine (Mission 018) with the "Dirty" Reality of Document Processing (Docling).
We will **MOCK** the Docling output to strictly verify the pipeline's tolerance and rejection logic before connecting the actual Python process.

## 2. SCOPE

### 2.1 The Mock Data ("Dirty Input")
Create `tests/mocks/docling_v2_export.json` containing:
- **Case A (Perfect):** A pristine BOQ table.
- **Case B (Real-world):** 
  - Merged headers (Docling artifact).
  - OCR typos (l vs 1).
  - Null descriptions.
  - Extra columns (noise).

### 2.2 The Integration Test
Create `tests/bridge_integration.rs`:
1. **Load JSON:** Parse mock file.
2. **Transform:** Map JSON → `TableTruth` (using `iron_table`).
3. **Execute:** Run `iron_engine::to_dataframe()` & `derive_project_truth()`.
4. **Assert:** Check `ProjectTruth` output against expected values.

### 2.3 Success Criteria
- [ ] Pipeline survives "Dirty" JSON (rejects bad rows, accepts good ones).
- [ ] Latency check: End-to-end processing < 100ms for test set.
- [ ] Rejection Log: Schema mismatches are correctly reported.

## 3. IMPLEMENTATION PLAN

### 3.1 Mock Construction
- Hand-craft JSON matching Docling v2 schema.
- Include edge cases intentionally.

### 3.2 Bridging Logic
- Ensure `iron_table` can deserialize the Mock JSON.
- Verify `TableTruth` construction correctness.

### 3.3 Execution
- Run `cargo test --test bridge_integration`.

---

## 4. NEXT STEPS (Post-Mission)
- Connect actual Python `docling` process via Stdin/Stdout (IPC).
