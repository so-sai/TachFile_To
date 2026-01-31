# TACHFILETO: THE UNIFIED UI CONSTITUTION (V1.0)

**Document Type:** Master Constitutional Specification
**Status:** 🛡️ SHIELDED & LOCKED
**Last Modified:** 2026-01-31
**Authority:** iron_coreVN Founder + Architecture

---

## 1. CORE PHILOSOPHY: TRUTH OVER UX
- **Identity:** TachFileTo is an **Inspection Instrument**, not an "app".
- **Primary Goal:** 100% traceabilty of data origin.
- **Visual Stance:** Brutalist, deterministic, zero-softening.
- **Forbidden:** No rounded corners, no shadows, no animations (except pulse for sync).

---

## 2. VISUAL GRAMMAR (BLOOMBERG SPECS)

### 2.1 Spatial Density
- **Row Height:** Strictly **32px**.
- **Grid Gap:** 0px (use borders for separation).
- **Padding:** 8px/12px horizontal only.

### 2.2 Scrollbar (THÔ)
- **Width:** Native **14px**.
- **Visibility:** Constant (Always visible).
- **Style:** Square-edged track (#E5E7EB), thumb (#9CA3AF).

### 2.3 Typography
- **Primary Font:** `Space Grotesk` or `SF Pro Display`.
- **Numbers:** Force `.tabular-nums` globally for all data grids.
- **Sizing:** 10px-13px strictly.

### 2.4 Color Palette (HIGH CONTRAST)
- **Summary Mode:** #00FF00 on Black (Green-on-Black).
- **Detail Mode:** #111827 on #F3F4F6 (Gray-900 on Gray-100).
- **Error/Rejection:** #B91C1C (Red-700) with #FEF2F2 (Red-50) background.

---

## 3. 4-PANEL ARCHITECTURE

### Panel 1: File Ledger [SỔ CÁI HỒ SƠ]
- **Status Tiers:** 
    - **SẠCH (CLEAN)**: Ingested & Verified.
    - **VẤN ĐỀ (TAINTED)**: Potential contradictions found.
    - **TỪ CHỐI (REJECTED)**: Structural failure / Corruption.

### Panel 2: Table Truth View [BẢN GỐC SỰ THẬT]
- **Requirement:** Virtualized Grid using `@tanstack/react-virtual`.
- **Scale:** Performance target 100,000+ rows @ 60fps.
- **Labels:** HỢP LỆ (ADMISSIBLE) / KHÔNG HỢP LỆ (INADMISSIBLE).

### Panel 3: Evidence Pane [BẰNG CHỨNG THỊ GIÁC]
- **Requirement:** Zero-latency crop display.
- **Source:** Direct 72DPI crop from PDF/Excel original file.
- **Metadata:** Show page number, bounding box, and extraction confidence.

### Panel 4: Discrepancy Summary [TỔNG HỢP SAI LỆCH]
- **Ticker:** Consistent vs Inconsistent vs Review counts.
- **Feature:** Cross-Source Contradiction Detection.

---

## 4. FORBIDDEN PATTERNS (HARD BLOCK)
- ❌ **No Spinners:** Use immediate text status or skeletal pulse.
- ❌ **No Fix Buttons:** UI is a microscope, not a screwdriver.
- ❌ **No Tooltips:** If data is important, display it clearly in the panel.
- ❌ **No Softening:** Never translate "REJECTED" as "Check again". Use "TỪ CHỐI".

---

## 5. IPC CONTRACT (APPENDIX)

### Command List:
- `get_file_ledger`: Ingested file list.
- `get_table_truth(file_id)`: Row-by-row truth verdicts.
- `get_evidence(cell_id)`: Source visual proofs.
- `get_discrepancy`: Session statistics.

---

## DOCUMENT AUTHORITY
This document is the **ORIGINAL LAW** of the TachFileTo UI. Any implementation found with a `border-radius > 0` is considered a constitutional violation.

**END OF CONSTITUTION**
