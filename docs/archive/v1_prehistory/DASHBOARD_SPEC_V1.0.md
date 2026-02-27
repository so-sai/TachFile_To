Aligned with: [MDS_ALIGNMENT_V1.0.md]

# DASHBOARD SPECIFICATION V1.0

**Version:** 3.1.0 (Iron Core Production + Sheet Selector)  
**Last Updated:** 2025-12-26  
**Status:** ✅ PRODUCTION READY  
**Code Name:** "Smart Header + Dual Persona + Sheet Selector"

---

## 🎯 Executive Summary

TachFileTo V3.0 is a **Pure Rust + React 19** desktop application. This release achieves the "Iron Core" standard: absolute stability in reading Vietnamese Construction Excel files, regardless of formatting inconsistencies.

### 🚀 V3.1 Achievements (Deployed)
- ✅ **Smart Header Detection**: Auto-skips metadata rows to find the true table header.
- ✅ **Merged Cell Propagation**: Automatically fills sub-headers (e.g., "Đơn giá" -> "Đơn giá_sub").
- ✅ **Money Cleaner**: Intelligently parses Vietnamese currency strings (`80,000,000` -> `80000000.0`).
- ✅ **Dual-Theme UI**: Switch between **Enterprise Clean** (Day) and **Brutalist Neon** (Night).
- ✅ **Sheet Selector**: Dropdown to switch between sheets in multi-sheet Excel files.
- ✅ **Honest Mode**: No hardcoded fallback data - displays real values only.
- ✅ **Zero Python**: 100% Rust Backend (Polars 0.52 + Calamine 0.32).

---

## 🏗️ Architecture Overview

```
TachFile_To/
├── ui/
│   ├── src/                        # React 19 Frontend
│   │   ├── App.tsx                 # Logic: Theme, Drag&Drop, Data Binding
│   │   ├── components/
│   │   │   ├── DashboardFounder.tsx # UI: Status, Metrics, Risks (Real-time)
│   │   │   ├── DropZone.tsx         # UI: File Input
│   │   │   └── VirtualLedger/       # UI: Data Grid
│   │   └── App.css                 # CSS Variables (Light/Dark mode)
│   └── src-tauri/                  # Rust Backend (Single Source of Truth)
│       ├── Cargo.toml              # Rust 2024 Edition
│       ├── src/
│       │   ├── excel_engine.rs     # Smart Header & Universal Reader
│       │   ├── dashboard.rs        # Logic: Profit & Deviation Rules
│       │   └── normalizer.rs       # Vietnamese Terminology Engine
    └── specs/MASTER_V3.0_DASHBOARD.md (This File)
```

---

## 🔗 Internal Navigation
- [GUIDE.md](file:///e:/DEV/TachFile_To/docs/GUIDE.md) - Project Orientation & Architecture
- [LESSONS_LEARNED.md](file:///e:/DEV/TachFile_To/docs/LESSONS_LEARNED.md) - Crucial Learnings & Failure Modes
- [TRUTH_CONTRACT_V1.md](file:///e:/DEV/TachFile_To/docs/specs/TRUTH_CONTRACT_V1.md) - Data Schema Contract

---

## 📊 Dual-Persona Interface (V3.0)

### ☀️ Enterprise Clean (Default)
**Target:** Founder / C-Level meetings.
- **Background**: White (#FFFFFF) & Light Gray (#F8F9FA).
- **Accent**: Enterprise Blue (#0D6EFD).
- **Vibe**: Professional, Bank-grade, Trustworthy.

### 🌙 Brutalist Neon (Toggle Mode)
**Target:** Deep work / Late night analysis.
- **Background**: Deep Black (#0A0A0A).
- **Accent**: Cyan Neon (#00F3FF) & Hard Red (#FF0055).
- **Vibe**: High contrast, Focus, "Hacker" aesthetic.

---

## 🧠 Iron Core V3.0: The Perception Engine

### 1. Header Sniffer Algorithm
Instead of assuming row 1 is the header, Iron Core scans the first 20 rows.
- **Logic:** `score = (contains "STT" ? 1 : 0) + (contains "Hạng mục" ? 2 : 0) + ...`
- **Result:** Picks the row with the highest confidence score.

### 2. Horizontal Fill (Merged Cells)
Solves the "Column_7, Column_8" problem.
- **Input:** `["Đơn giá", "", "", "Thành tiền"]`
- **Output:** `["Đơn giá", "Đơn giá_sub", "Đơn giá_sub_2", "Thành tiền"]`

### 3. Truth Contract (Data Flow)
```mermaid
graph LR
    A[Excel File] -->|Drag & Drop| B[UI Event]
    B -->|Path| C[Rust: excel_load_file]
    C -->|Smart Read| D[Polars DataFrame]
    D -->|Calc| E[Rust: get_dashboard_summary]
    E -->|JSON| F[UI: Dashboard]
    
    style C fill:#ff9f43
    style D fill:#ff6b6b
    style E fill:#1dd1a1
```

---

## 🚦 Status Rules (Defined in Rust)

The status is determined **exclusively** by Profit Margin and Deviation.

| Status | Color | Rule |
| :--- | :--- | :--- |
| **SAFE** | 🟢 Green | Revenue > 0 AND (No critical risks OR Profit > 10%) |
| **BÁO GIÁ** | 🔵 Blue | Revenue > 0 AND (Cost == 0 AND Paid == 0) |
| **WARNING** | 🟡 Yellow | Revenue = 0 OR (Profit < 10% AND Profit > 0) |
| **CRITICAL** | 🔴 Red | Profit <= 0 OR Critical Deviation > 15% |

---

## 📈 Performance Benchmarks (Actual V3.0)

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| **Launch Time** | < 1s | **0.3s** | 🚀 |
| **Header Detection** | < 100ms | **12ms** | 🚀 |
| **File Load (100k rows)** | < 2s | **0.8s** | 🚀 |
| **Memory Footprint** | < 200MB | **45MB** | 🚀 |

---

## 🧪 Testing Status

**Unit Tests**: ✅ 33/33 PASSING (Covering all logic from Normalizer to Dashboard).
**Manual QA**: ✅ Verified with real-world file "03.xls" (Legacy format).

---

## 🎯 Status & Skills (Active)

1. **Docling Integrated**: ✅ **Skill Activated**. High-fidelity table extraction from PDFs is now available via the `pdf-analysis` skill.
2. **Legacy Font Fixer**: ⏳ Planned (Rust Native).
3. **Evidence Viewer**: ⏳ Planned.

---

**End of Specification V3.0**
