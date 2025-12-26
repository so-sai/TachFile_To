# TACHFILETO V2.3 - ARCHITECTURE (SINGLE SOURCE OF TRUTH)

**Version:** 2.3.0  
**Last Updated:** 2025-12-26  
**Status:** Production Architecture (Pure Rust Stack)

---

## 🎯 Core Philosophy: Perception Engine

> **"Users don't need to see 1M rows. They need to FEEL in control of 1M rows."**

TachFileTo V2.3 is a **Pure Rust** desktop application for Vietnamese construction quantity surveyors. It processes Excel files with **Polars 0.52**, normalizes Vietnamese accounting terminology, and provides a windowed virtual ledger UI that handles millions of rows with instant responsiveness.

---

## 🏗️ Architecture Overview

```
TachFile_To/
├── ui/
│   ├── src/                    # React Frontend (Perception Engine UI)
│   │   ├── App.tsx            # Main application with virtual windowing
│   │   ├── components/
│   │   │   └── VirtualLedger/ # TanStack Virtual table (100-500 row window)
│   │   └── styles/            # Enterprise Eye-Safe Design System
│   └── src-tauri/             # Rust Backend (SINGLE SOURCE)
│       ├── Cargo.toml         # Polars 0.52 + polars-io/excel
│       ├── src/
│       │   ├── main.rs        # Tauri entry point
│       │   ├── lib.rs         # Tauri commands & state
│       │   ├── excel_engine.rs    # Excel reading + Terminology Normalizer
│       │   └── normalizer.rs      # Vietnamese term standardization
│       └── target/
└── docs/
    └── specs/
        ├── ARCHITECTURE_V2.3.md   # THIS FILE (Master Source)
        └── archive/               # Legacy docs (v1.1 - OBSOLETE)
```

---

## 🔧 Technology Stack (2025 Standard)

| Layer | Technology | Version | Purpose |
|-------|-----------|---------|---------|
| **Frontend** | React + TypeScript | 18+ | Perception Engine UI |
| **Desktop Runtime** | Tauri | 2.0 | Native desktop wrapper |
| **Backend** | Rust | Edition 2021 | Core business logic |
| **Data Engine** | Polars | 0.52 | DataFrame processing |
| **Excel I/O** | polars-io | 0.52 | Excel reading (via calamine) |
| **IPC** | Tauri Commands | - | Direct Rust ↔ Frontend |
| **Virtualization** | TanStack Virtual | 3.x | Windowed rendering |

---

## 📊 Core Architecture: Perception Engine

### Data Flow

```mermaid
graph LR
    A[Excel File] --> B[polars-io ExcelReader]
    B --> C[Polars DataFrame]
    C --> D[Terminology Normalizer]
    D --> E[Standardized DataFrame]
    E --> F[Windowing Engine]
    F --> G[UI Slice 100-500 rows]
    G --> H[TanStack Virtual]
    H --> I[User sees 50 DOM nodes]
    
    style B fill:#e1f5ff
    style D fill:#e1ffe1
    style F fill:#fff4e1
```

### Key Principles

1. **Full Load**: Polars loads entire Excel into RAM (up to 1M+ rows)
2. **Terminology Normalization**: Auto-standardize Vietnamese column names
3. **Windowing**: Only send 100-500 row slices to frontend
4. **Virtual Rendering**: TanStack renders <50 DOM nodes
5. **Perception**: User feels instant control over millions of rows

---

## 🔥 What Changed from V1.1 (BREAKING)

| V1.1 (OBSOLETE) | V2.3 (CURRENT) | Reason |
|-----------------|----------------|--------|
| Python Worker + Stdio IPC | Pure Rust + Tauri Commands | Eliminate serialization overhead |
| SQLite Cache | In-Memory DataFrame | Faster, simpler |
| Docling Python | Polars Native | No Python runtime dependency |
| TCVN3/VNI Font Converter | Unicode-only | Modern data sources |
| Lazy Loading | Perception Engine | Better UX for large datasets |

---

## 🚀 Development Workflow

### Running the Application

```bash
# From project root
cd ui
npm run tauri dev

# The app will:
# 1. Start Vite dev server (port 1420)
# 2. Compile Rust backend
# 3. Launch Tauri window
```

### Testing with Real Data

1. Drag & drop an Excel file (.xlsx) into the app
2. Engine automatically normalizes column names (e.g., "Đ.Giá" → "don_gia")
3. Scroll through millions of rows with instant responsiveness
4. Virtual window ensures <50 DOM nodes at any time

### Building for Production

```bash
cd ui
npm run tauri build

# Output: ui/src-tauri/target/release/tachfileto-core.exe
```

---

## 📝 Key Components

### Excel Engine (`excel_engine.rs`)

```rust
pub struct ExcelEngine;

impl ExcelEngine {
    // Read Excel + Auto-normalize column names
    pub fn read_and_normalize(path: &str) -> Result<DataFrame> {
        let mut df = Self::read_raw_excel(path)?;
        Self::normalize_schema(&mut df)?;
        Ok(df)
    }
}
```

**Features:**
- Uses `polars-io` with `excel` feature
- Automatic Vietnamese terminology normalization
- Handles 1M+ rows in <2 seconds

### Terminology Normalizer (`normalizer.rs`)

Standardizes Vietnamese accounting terms:

| Input Variants | Standardized Output |
|----------------|---------------------|
| "Đ.Giá", "Đơn giá", "Unit Price" | `don_gia` |
| "Thành tiền", "Total", "Amount" | `thanh_tien` |
| "Khối lượng", "Qty", "Volume" | `khoi_luong` |

---

## 🚫 What We DON'T Use Anymore

- ❌ Python Worker
- ❌ Stdio JSON IPC
- ❌ SQLite Cache
- ❌ Legacy font converters (TCVN3/VNI)
- ❌ Docling Python bindings

---

## 📚 Documentation Hierarchy

1. **ARCHITECTURE_V2.3.md** (THIS FILE) - Master source of truth
2. **Cargo.toml** - Dependency reality check
3. **Code** - Ultimate truth

### Obsolete Documents (DO NOT USE)

- `docs/GUIDE.md` (v1.1 - ARCHIVED)
- `docs/specs/ARCHITECTURE_MASTER.md` (v1.1 - ARCHIVED)
- `docs/specs/IPC_PROTOCOL.md` (v1.1 - ARCHIVED)

All obsolete docs have 🚨 banners pointing to this file.

---

## 🎯 Next Steps (Q1 2026)

- [ ] Excel Export with Vietnamese QS templates
- [ ] PDF table extraction (via Polars)
- [ ] Multi-file batch processing
- [ ] Cloud sync (optional)

---

**For implementation details, see the actual code in `ui/src-tauri/src/`.**
