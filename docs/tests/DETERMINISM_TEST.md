# 🔒 DETERMINISM TEST CONTRACT

**ID:** DTC-PDF-001  
**Status:** PROPOSED – PENDING ARCHITECT SIGN-OFF  
**Phase:** 0 (CODE = 0)  
**Scope:** Iron Core PDF Intelligence (Infrastructure Layer)

---

## 1. PURPOSE

This document defines the **determinism requirements** for the PDF Extraction subsystem.

> [!CAUTION]
> **Any non-deterministic behavior is a CRITICAL BUG.**  
> Non-determinism in financial data extraction is unacceptable.

---

## 2. CORE PRINCIPLE

> **Same input → Same output → Same hash. Always.**
>
> Across runs. Across threads. Across machines. Across OS.

---

## 3. HASH PIPELINE

### 3.1 What Gets Hashed

| Stage | Content | Hash Algorithm |
|:------|:--------|:---------------|
| Input | Source PDF file | SHA-256 |
| Output | Canonical JSON (keys sorted) | SHA-256 |
| Log | Execution log (timestamps stripped) | SHA-256 |

### 3.2 Hash Normalization Rules

Before hashing output JSON:
- Sort all object keys alphabetically
- Remove whitespace variations
- Normalize decimal precision (2 digits for money, 4 for quantity)
- Strip execution timestamps

---

## 4. 10-RUN IDENTITY TEST

### 4.1 Protocol

```
Input: contract_sample.pdf
Runs: 10 consecutive, same process
Output: 10 JSON files

For each JSON:
  1. Normalize (sort keys, strip timestamps)
  2. Compute SHA-256
  3. Compare to first run's hash

Expected: 10/10 identical hashes
```

### 4.2 Verdict

| Result | Action |
|:-------|:-------|
| 10/10 identical | ✅ PASS |
| 9/10 identical | ❌ REJECT |
| <9/10 identical | ❌ REJECT + REQUIRE ROOT CAUSE |

---

## 5. CROSS-ENVIRONMENT TESTS

### 5.1 OS Variance Test

Same PDF must produce identical output on:

| Environment | Status |
|:------------|:-------|
| Windows 11 (native) | REQUIRED |
| Linux (musl) | REQUIRED |
| macOS (ARM) | OPTIONAL |

### 5.2 Thread Count Variance Test

Same PDF must produce identical output with:

| Thread Config | Status |
|:--------------|:-------|
| Single-threaded | REQUIRED |
| 4 threads | REQUIRED |
| 12 threads | REQUIRED |

---

## 6. ISOLATION TESTS

### 6.1 Native vs Sandboxed

Test MUST pass in both:

| Environment | Description |
|:------------|:------------|
| Native | `cargo test` directly |
| Isolated | Container with resource limits (2GB RAM, 2 CPU) |

### 6.2 Cold vs Warm State

| State | Requirement |
|:------|:------------|
| Cold (first run) | Output must match warm runs |
| Warm (runs 2-10) | All must be identical |

---

## 7. FORBIDDEN BEHAVIORS

| Behavior | Impact | Action |
|:---------|:-------|:-------|
| Random seed in extraction | Non-deterministic | **REJECT** |
| Timestamp in output values | Non-deterministic | **REJECT** |
| Thread-dependent ordering | Non-deterministic | **REJECT** |
| Floating-point rounding variance | Non-deterministic | **REJECT** |
| Locale-dependent formatting | Non-deterministic | **REJECT** |

---

## 8. HASH LOG FORMAT

Every determinism test MUST output:

```json
{
  "test_id": "DTC-PDF-001-RUN-001",
  "input_hash": "sha256:abc123...",
  "runs": [
    {
      "run": 1,
      "output_hash": "sha256:def456...",
      "log_hash": "sha256:ghi789..."
    },
    ...
  ],
  "all_identical": true,
  "verdict": "PASS"
}
```

---

## 9. FINAL CLAUSE

> **Determinism is the foundation of trust.**
>
> If the same input can produce different outputs, the system cannot be trusted for financial data.
> This is not negotiable.

---

**Architect Sign-off:** ________________________  
**Date:** ________________________
