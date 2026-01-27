# ⚡ PERFORMANCE GATE TEST CONTRACT

**ID:** PGT-PDF-001  
**Status:** PROPOSED – PENDING ARCHITECT SIGN-OFF  
**Phase:** 0 (CODE = 0)  
**Scope:** Iron Core PDF Intelligence (Infrastructure Layer)

---

## 1. PURPOSE

This document defines the **performance kill thresholds** for the PDF Extraction subsystem.

> [!CAUTION]
> Exceeding ANY threshold results in **IMMEDIATE REJECTION**.  
> No exceptions. No "optimize later".

---

## 2. HARDWARE BASELINE

All benchmarks MUST be run on hardware matching or exceeding:

| Component | Specification |
|:----------|:--------------|
| CPU | Intel i5-11400H (12 threads) |
| RAM | 8GB allocated (not total) |
| Storage | SSD NVMe |
| GPU | ❌ NOT USED (CPU-only baseline) |
| OS | Windows 11 / Linux (musl) |

---

## 3. SLA THRESHOLDS

### 3.1 Kill Thresholds

| Metric | Target | Hard Limit | Violation Action |
|:-------|:-------|:-----------|:-----------------|
| **Cold Start** (Init Runtime) | ≤ 3s | ≤ 5s | REJECT PR |
| **Warm Parse** (100 pages) | ≤ 0.8s | ≤ 1.0s | REJECT PR |
| **Peak RAM** | ≤ 1.5× input | ≤ 2× input | REJECT PR |
| **Memory Leak** (100 iterations) | 0 MB | ≤ 5MB delta | REJECT PR |

### 3.2 Benchmark Protocol

```
Test File: contract_100pages.pdf (15MB)
Iterations: 100 consecutive runs
Measurement: 
  - First run = Cold Start
  - Runs 2-100 = Warm Parse (avg)
  - Peak RSS after run 100 vs run 1
```

---

## 4. REJECTION MATRIX

| Scenario | Threshold | Action |
|:---------|:----------|:-------|
| Cold start > 5.0s | HARD LIMIT | **REJECT** |
| Cold start > 10.0s | CRITICAL | **REJECT + BLOCK MERGE** |
| Warm parse > 1.0s | HARD LIMIT | **REJECT** |
| Warm parse > 3.0s | CRITICAL | **REJECT + REQUIRE REDESIGN** |
| Peak RAM > 2× input | HARD LIMIT | **REJECT** |
| Memory leak > 5MB | HARD LIMIT | **REJECT** |

---

## 5. BENCHMARK OUTPUT REQUIREMENTS

### 5.1 Audit Trail

Every benchmark run MUST output:

```json
{
  "benchmark_id": "uuid",
  "timestamp": "RFC3339",
  "commit_hash": "git-sha",
  "hardware": {
    "cpu": "i5-11400H",
    "ram_allocated_mb": 8192,
    "os": "Windows 11"
  },
  "results": {
    "cold_start_ms": 2345,
    "warm_parse_avg_ms": 756,
    "warm_parse_p99_ms": 890,
    "peak_ram_mb": 245,
    "memory_delta_mb": 2.3
  },
  "verdict": "PASS"
}
```

### 5.2 File Naming

```
bench_results_{YYYYMMDD}_{commit_hash_short}.json
```

---

## 6. COMPARISON REQUIREMENTS

### 6.1 Regression Detection

Each new build MUST compare against previous benchmark:

| Metric | Regression Threshold | Action |
|:-------|:---------------------|:-------|
| Cold start | > 10% slower | WARNING |
| Cold start | > 25% slower | REJECT |
| Warm parse | > 5% slower | WARNING |
| Warm parse | > 15% slower | REJECT |
| Peak RAM | > 10% higher | WARNING |
| Peak RAM | > 25% higher | REJECT |

---

## 7. TEST CORPUS FOR BENCHMARK

| ID | File | Pages | Size | Purpose |
|:---|:-----|:------|:-----|:--------|
| B1 | `contract_50pages.pdf` | 50 | 5MB | Baseline |
| B2 | `contract_100pages.pdf` | 100 | 15MB | Stress |
| B3 | `scanned_100pages.pdf` | 100 | 50MB | OCR stress |

---

## 8. FINAL CLAUSE

> **Iron Core performance is non-negotiable.**
>
> If the system cannot meet these thresholds, it has no place in production.
> "Optimize later" is not a valid response.

---

**Architect Sign-off:** ________________________  
**Date:** ________________________
