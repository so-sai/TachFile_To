# LATENCY BUDGET: IRON TRUTH PIPELINE
**Status:** ENFORCED
**Applies to:** Mission 017+  
**Target Hardware:** Standard Desktop (No GPU required)

---

## 1. GLOBAL LATENCY CEILING
**Hard Limit:** 2,000ms (2 Seconds) per 100 Pages.
**Constraint:** Linear scaling O(n) or better.

Any build exceeding this budget is considered a **FAILED BUILD**.

---

## 2. STAGE BREAKDOWN

### Stage 1: File Load & Text Extraction (PyMuPDF)
- **Role:** IO / Native Extraction
- **Budget:** 150ms total
- **Typical:** 50-120ms
- **Note:** Must remain IO-bound.

### Stage 2: Structure Detection (Docling v2)
- **Role:** Deep Table Logic / Layout Analysis
- **Budget:** 15ms per page
- **Typical:** 8-15ms/page
- **Budget (100 pages):** 1,500ms
- **Note:** This is the primary compute cost. Parallelization allowed.

### Stage 3: Python → Rust Boundary (IPC)
- **Role:** Serialization / Transport
- **Budget:** 100ms total
- **Typical:** 30-60ms
- **Note:** Zero-copy where possible.

### Stage 4: Iron Core Logic (Validation & Normalization)
- **Role:** Contract Enforcement / Normalization
- **Budget:** 150ms total
- **Typical:** < 100ms
- **Note:** Deterministic logic is cheap. No excuses involved.

### Stage 5: Polars Frame Construction
- **Role:** Data Engine Loading
- **Budget:** 200ms total
- **Typical:** 50-150ms
- **Note:** Explicit schema construction only.

---

## 3. ENFORCEMENT POLICY

1. **Performance Regression:** - Any PR increasing Stage 4 or 5 latency by >10% must be rejected unless justified by major feature.
2. **Docling Constraints:** - If Docling exceeds 20ms/page, the configuration must be tuned (reduce resolution/complexity) rather than accepting the slowdown.
3. **Memory:** - Peak RAM < 500MB during processing of 100 pages.

---
**Verdict:** "The glass panel must respond instantly, even if the engine is heavy."
