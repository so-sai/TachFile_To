# 📑 PDF EXTRACTION ACCEPTANCE TEST CONTRACT

**ID:** ATC-PDF-001  
**Status:** PROPOSED – PENDING ARCHITECT SIGN-OFF  
**Phase:** 0 (CODE = 0)  
**Scope:** Iron Core PDF Intelligence (Infrastructure Layer)

---

## 1. PURPOSE (DEFINITION OF DONE)

This document defines the **non-negotiable acceptance criteria** for the PDF Extraction subsystem.

> [!CAUTION]
> If any criterion in this document is violated, the implementation is considered **FAILED**, regardless of feature completeness.

PDF Intelligence is **infrastructure**, not an AI feature.

---

## 2. INPUT CORPUS SPECIFICATION

### 2.1 Mandatory Test Corpus

All implementations MUST be validated against the following corpus:

| ID | Category | Description |
|:---|:---------|:------------|
| L1 | Construction Contract | 1× PDF, 30–50 pages, mixed text + tables |
| L2 | Payment Statement | 1× PDF, 10–20 pages, dense tabular data |
| L3 | Scanned Document | 1× PDF, OCR-required (Vietnamese + numbers) |
| L4 | Corrupted File | 1× partially corrupted PDF |

### 2.2 Input Constraints

| Constraint | Value |
|:-----------|:------|
| Max file size | ≤ 50 MB |
| Language | Vietnamese (UTF-8), numeric-heavy |
| Network access | ❌ FORBIDDEN during processing |

---

## 3. OUTPUT CONTRACT (CANONICAL JSON)

### 3.1 Output Format

- **Single canonical JSON document** per input PDF
- UTF-8 encoded
- Stable key ordering (alphabetical)

### 3.2 Required Top-Level Fields

```json
{
  "document_id": "string",
  "source_file_hash": "sha256",
  "extraction_timestamp": "RFC3339",
  "pages": number,
  "items": [ ... ],
  "tables": [ ... ],
  "errors": [ ... ]
}
```

### 3.3 Typed Fields (Non-Negotiable)

| Field Type | Format | Example |
|:-----------|:-------|:--------|
| Monetary values | `decimal(18,2)` | `18500000.00` |
| Quantities | `decimal(18,4)` | `1200.0000` |
| Dates | ISO-8601 | `2026-01-12` |

> [!WARNING]
> **No free-form numeric strings allowed.**  
> `"18,500,000 VNĐ"` → ❌ REJECTED  
> `18500000.00` → ✅ ACCEPTED

---

## 4. DETERMINISM REQUIREMENT

### 4.1 Deterministic Rule

> **Same input → same output → same hash. Always.**

### 4.2 Determinism Test Protocol

1. Execute extraction **10 consecutive runs** on the same input
2. Compute SHA-256 of:
   - Output JSON (normalized, keys sorted)
3. **All 10 hashes MUST be identical**

| Result | Action |
|:-------|:-------|
| 10/10 identical | ✅ PASS |
| <10/10 identical | ❌ REJECT |

---

## 5. FAILURE MODES (EXPLICIT)

| Scenario | Expected Behavior | Error Code |
|:---------|:------------------|:-----------|
| OCR failure | Partial output + structured error | `ERR_OCR_FAILED` |
| Table merge failure | Preserve raw cells + error annotation | `ERR_TABLE_MERGE` |
| Corrupted PDF | Fail-fast, no panic, no undefined state | `ERR_PDF_CORRUPT` |
| Unsupported format | Deterministic error message | `ERR_UNSUPPORTED` |
| OCR low confidence | Warning + coordinates of suspect region | `WARN_OCR_LOW_CONF` |

> [!CAUTION]
> **Silent failure is STRICTLY FORBIDDEN.**  
> Every error must produce a traceable, structured response.

---

## 6. TABLE EXTRACTION REQUIREMENTS

### 6.1 Structural Integrity

- MUST preserve Parent-Child relationship of header rows
- MUST handle merged cells (horizontal & vertical)
- MUST handle nested tables (table within table)

### 6.2 Field Integrity

| Field | Type | Violation Action |
|:------|:-----|:-----------------|
| `unit_price` | Number | REJECT if String |
| `quantity` | Number | REJECT if String |
| `amount` | Number | REJECT if String |

---

## 7. ACCEPTANCE VERDICT

The system is considered **ACCEPTED** only if:

- [ ] All input corpus files produce valid canonical JSON
- [ ] All determinism checks pass (10/10 identical)
- [ ] No undefined / free-form numeric output exists
- [ ] All failure modes produce structured error codes
- [ ] Schema validation passes against `schemas/extraction_v1.json`

---

## 8. FINAL CLAUSE

> This document is a **binding acceptance contract**.
>
> Implementation details (Rust, Python, PyO3, PyOxidizer) are **irrelevant** if these criteria are not met.

---

**Architect Sign-off:** ________________________  
**Date:** ________________________
