# 🗺️ TEST STRATEGY

**ID:** TS-PDF-001  
**Status:** PROPOSED  
**Phase:** 0 (CODE = 0)

---

## 1. PURPOSE

Maps Test Contracts to future code modules.

> No code shall exist without a corresponding test contract.

---

## 2. CONTRACT → CODE MAPPING

| Test Contract | Target Module | Test Files |
|:--------------|:--------------|:-----------|
| `PDF_EXTRACTION_ACCEPTANCE_TEST.md` | `lib.rs::extract_pdf()` | `tests/acceptance.rs` |
| `PERFORMANCE_GATE_TEST.md` | `benches/extraction.rs` | `benches/` |
| `DETERMINISM_TEST.md` | `tests/determinism.rs` | `tests/determinism.rs` |

---

## 3. TDD PHASES

### Phase 0: Contracts Only (Current)
- [x] `PDF_EXTRACTION_ACCEPTANCE_TEST.md`
- [x] `PERFORMANCE_GATE_TEST.md`
- [x] `DETERMINISM_TEST.md`
- [ ] Architect Sign-off

### Phase 1: Policy Update
- [ ] Add §1.3 TDD Enforcement to policy
- [ ] Delete invalid skeleton code

### Phase 2: Red Phase
- [ ] Write `tests/integration_test.rs` (MUST FAIL)
- [ ] Write `tests/acceptance.rs` (MUST FAIL)
- [ ] Write `tests/determinism.rs` (MUST FAIL)

### Phase 3: Green Phase
- [ ] Implement minimal code to pass tests
- [ ] No feature creep

---

## 4. TEST CORPUS LOCATION

```
tests/
├── corpus/
│   ├── contract_50pages.pdf      # Baseline
│   ├── contract_100pages.pdf     # Stress
│   ├── scanned_100pages.pdf      # OCR stress
│   ├── corrupted.pdf             # Failure mode
│   └── contract_sample.pdf       # Determinism test
└── expected/
    ├── contract_50pages.json     # Ground truth
    ├── contract_100pages.json
    └── contract_sample.json
```

---

## 5. CI/CD INTEGRATION

### Required Gates

| Gate | Trigger | Action on Fail |
|:-----|:--------|:---------------|
| Acceptance Tests | Every PR | Block merge |
| Performance Benchmark | Every PR | Block merge if regression |
| Determinism Test | Every PR | Block merge |

### Gate Order

```
1. cargo fmt --check
2. cargo clippy
3. cargo test (acceptance)
4. cargo test (determinism)
5. cargo bench (performance)
6. Architect review (manual)
```

---

## 6. SIGN-OFF REQUIREMENTS

Before ANY code is written:

- [ ] Acceptance Contract signed
- [ ] Performance Contract signed  
- [ ] Determinism Contract signed
- [ ] Test corpus files prepared
- [ ] Expected output (ground truth) prepared

---

**Architect Sign-off:** ________________________  
**Date:** ________________________
