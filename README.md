# TachFileTo V2.5 - Founder's Eye

**Pure Rust + React Desktop Application for Vietnamese Construction QS**

[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)
[![Polars](https://img.shields.io/badge/Polars-0.52-blue.svg)](https://pola.rs/)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-green.svg)](https://tauri.app/)

---

## 🎯 What is TachFileTo?

TachFileTo is a **deterministic decision engine** for construction project founders and quantity surveyors in Vietnam. It processes Excel/PDF data and provides:

- 🚦 **Founder Dashboard**: RED/YELLOW/GREEN status with zero ambiguity
- 📊 **Virtual Ledger**: Handle 1M+ rows with instant scrolling
- 🇻🇳 **Vietnamese-First**: 100% localized for Vietnamese construction industry
- ⚡ **Pure Rust Backend**: No Python, no dependencies, just speed

---

## 🚀 Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (2021 edition or later)
- [Node.js](https://nodejs.org/) (v18+)
- [pnpm](https://pnpm.io/) or npm

### Development

```bash
# Clone the repository
git clone https://github.com/YOUR_USERNAME/TachFile_To.git
cd TachFile_To

# Install frontend dependencies
cd ui
npm install

# Run in development mode
npm run tauri dev
```

### Build for Production

```bash
cd ui
npm run tauri build

# Output: ui/src-tauri/target/release/tachfileto-core.exe
```

---

## 📚 Architecture

```
TachFile_To/
├── ui/
│   ├── src/                    # React Frontend
│   │   ├── App.tsx            # Tab Navigation
│   │   └── components/
│   │       ├── DashboardMockup.tsx    # Founder Dashboard
│   │       └── VirtualLedger/         # QS Data View
│   └── src-tauri/             # Rust Backend
│       ├── src/
│       │   ├── excel_engine.rs    # Polars + Calamine
│       │   ├── normalizer.rs      # Vietnamese terms
│       │   └── dashboard.rs       # Business rules
│       └── Cargo.toml
└── docs/
    └── specs/
        ├── MASTER_V2.5_DASHBOARD.md   # Architecture
        └── TRUTH_CONTRACT_V1.md       # API Schema
```

**Key Technologies**:
- **Backend**: Rust 2021 + Polars 0.52 + Calamine 0.32
- **Frontend**: React 18 + TypeScript + TanStack Virtual
- **Desktop**: Tauri 2.0

---

## 🎨 Features

### Founder Dashboard (V2.5)
- **Deterministic Status**: SAFE/WARNING/CRITICAL based on hard rules
- **Financial Overview**: Contract value, payments, projected profit
- **Risk Prioritization**: Top 5 risks sorted by cost impact
- **Action Items**: Prioritized tasks with deadlines

### Data View (QS/PM)
- **Virtual Ledger**: Scroll through millions of rows
- **Auto-Normalization**: Vietnamese column names standardized
- **Excel-like UX**: Native scrollbar, 32px rows, tabular numbers

---

## 📖 Documentation

- [Master Specification](docs/specs/MASTER_V2.5_DASHBOARD.md) - Complete architecture
- [Truth Contract](docs/specs/TRUTH_CONTRACT_V1.md) - API schema
- [Walkthrough](docs/walkthrough.md) - Implementation guide

---

## 🗺️ Roadmap

- ✅ **V2.5** (Current): Founder Dashboard + Tab Navigation
- ⏳ **V2.6** (Q1 2026): PDF extraction (Docling integration)
- ⏳ **V2.7** (Q1 2026): Evidence Viewer (visual verification)
- ⏳ **V2.8** (Q2 2026): Export to Word/PDF

See [MASTER_V2.5_DASHBOARD.md](docs/specs/MASTER_V2.5_DASHBOARD.md) for complete roadmap.

---

## 🛡️ Philosophy

> **"Dashboard không phải để xem. Dashboard là để quyết định."**

TachFileTo follows the **Iron Core** principle:
- UI has **zero logic** - only renders what Rust declares as truth
- All calculations are **deterministic** and reproducible
- Vietnamese construction terminology is **first-class**

---

## 📝 License

MIT License - See [LICENSE](LICENSE) for details

---

## 🤝 Contributing

This project is currently in active development for V2.5. Contributions welcome after initial release.

---

**Built with ❤️ for Vietnamese construction industry**
