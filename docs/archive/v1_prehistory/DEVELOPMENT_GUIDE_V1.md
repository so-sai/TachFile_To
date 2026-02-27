# 🛠️ DEVELOPMENT GUIDE V1.0 – TachFileTo Core

**Status:** ✅ SINGLE SOURCE OF TRUTH (Mission 016+ Compliance)  
**Scope:** Architecture, Build, FFI Debugging, and Contribution guidelines.

---

## 📂 1. STRATEGIC ORIENTATION

### Core Philosophy
TachFileTo follows the **ANTI_GRAVITY.md** principles:
- **Stateless**: No file paths stored in persistent state.
- **Least Privilege**: Zero unnecessary permissions.
- **Zero Business Logic in UI**: React only renders what Iron Core declares as truth.
- **Deterministic**: Algorithms only. No "AI-guessing" in the core engine.

### Technology Stack
- **Frontend**: React 19 + TypeScript
- **Desktop Runtime**: Tauri 2.0
- **Logic Engine**: Rust (Edition 2024)
- **Data Engine**: Polars 0.52 (Mission 017 target)
- **Table Sensor**: Docling v2 (Python/Bridge) - *Note: pypdfium2 as fallback backend*

---

## 🏗️ 2. BUILD INSTRUCTIONS

### Requirements
- **Hardware**: 8GB+ RAM, 5GB free disk.
- **Software**:
  - Rust 1.92+ (`rustup update stable`)
  - Python 3.14t (**Free-threaded binaries REQUIRED**)
  - Node.js 20+ (LTS)
  - `uv` (Fast pip replacement)

### Quick Start
```powershell
# 1. Setup Python venv (No-GIL)
python3.14t -m venv .venv_nogil
.\.venv_nogil\Scripts\Activate.ps1
uv pip install -e .
uv pip install docling[pypdfium2] polars

# 2. Setup UI
cd ui
npm install

# 3. Development Mode
npm run tauri dev
```

### Production Build
```powershell
cd ui
npm run tauri build -- --release
# Output: ui/src-tauri/target/release/tachfileto.exe
```

---

## 🤝 3. CONTRIBUTION & PR PROTOCOL

### Upstream Integrity
TachFileTo maintains a **Community-First** approach. However, due to the wontfix status of PyMuPDF #4875:
- **Primary Backend**: `pypdfium2` (Native cp314t support).
- **Secondary**: Custom Rust-bridged MuPDF static builds.

### Pull Request Rules
- **No Style Changes**: Do not refactor code for "cleanliness" unless approved.
- **No New Dependencies**: Every crate added needs specific justification.
- **100% Vietnamese UI**: User-facing text must be in Vietnamese.
- **TDD Requirement**: Logic changes must include matching tests in `docs/tests/` or equivalent.

---

## 🔍 4. ADVANCED FFI DEBUGGING (WINDOWS MSVC)

When encountering "Unresolved External Symbol" (LNK2019):

### Diagnostic Step
Use `dumpbin /SYMBOLS` to verify the library actually contains the symbol and check for name mangling or LTCG issues.

### Common Surgical Fixes
1. **LTCG Issue**: MuPDF often ships with `/GL` (LTCG). Rust linker cannot see inside.
   - **Fix**: Rebuild C Lib with `/p:WholeProgramOptimization=false`.
2. **Runtime Conflict**: Ensure the C library and Rust use the same MSVC Runtime (Standardize on `/MD`).
3. **Missing extern "C"**: Ensure FFI signatures are wrapped correctly to prevent C++ mangling.

---

## 📊 5. PERFORMANCE TARGETS

| Metric | Target | Hard Limit |
|:-------|:-------|:-----------|
| Cold Start | ≤ 3s | ≤ 5s |
| Throughput | ≥ 25 items/sec | ≥ 20 items/sec (Mission 015 benchmark) |
| Memory | ≤ 500MB | ≤ 1GB (100-page processing) |
| Binary Size | ≤ 70MB | ≤ 100MB |

---

## 🤖 6. AI AGENT GOVERNANCE

Any AI Agent participating in this project MUST:
1. **Read `docs/specs/TRUTH_*`** before proposing data changes.
2. **Respect `docs/archive/`**: Do not use archived missions as current context.
3. **Fail-Fast**: Propose rejection paths (ERR) for ambiguous data.

---
**Version History:**
- 1.0.0 (2026-01-31): Unified orientation, build, PR, and debug guides.
- Archived: `GUIDE.md`, `build_guide.md`, `PR_guide.md`.
