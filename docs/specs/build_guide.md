# 🛠️ BUILD GUIDE – Xây Dựng TachFileTo Portable EXE

**Version:** 1.0 – Alpha RC1 (Tháng 1/2026)  
**Status:** APPROVED  
**Scope:** Hướng dẫn build TachFileTo từ source thành portable .exe (~50-70MB)

---

## 1. TỔNG QUAN

Đây là hướng dẫn chi tiết để build **TachFileTo** từ GitHub thành một portable `.exe` file có thể chạy offline, không cần cài đặt dependencies ngoài.

**Kết quả cuối cùng:**
- 📦 `.exe` ~50-70MB (sau UPX compression)
- 🔒 Embed Python 3.14 + Docling + Polars
- ⚡ Chạy offline, deterministic
- 🦀 Rust 1.92 (Edition 2024) + Python 3.14t (No-GIL)

---

## 2. YÊU CẦU HỆ THỐNG

### 2.1 Hardware Tối Thiểu

| Component | Tối thiểu | Khuyến nghị |
|:----------|:----------|:------------|
| CPU | 4 cores | 8+ cores (build nhanh hơn) |
| RAM | 8GB | 16GB |
| Storage | 5GB free | 10GB free (cache) |
| OS | Windows 10 21H2 | Windows 11 |

### 2.2 Software Dependencies

| Tool | Version | Cài đặt |
|:-----|:--------|:--------|
| **Rust** | 1.92+ | `rustup update stable` |
| **Python** | 3.14.2t | [python.org](https://python.org) (tick "free-threaded") |
| **Node.js** | 20+ LTS | [nodejs.org](https://nodejs.org) |
| **uv** | latest | `pip install uv` |
| **Git** | latest | [git-scm.com](https://git-scm.com) |

> [!IMPORTANT]
> **Python 3.14:** Khi cài đặt, PHẢI tick "free-threaded binaries" để có No-GIL support.

---

## 3. TẢI VÀ CẤU HÌNH REPO

### 3.1 Clone Repository

```bash
# Clone từ fork của bạn
git clone https://github.com/<your-username>/TachFileTo.git
cd TachFileTo

# Thêm upstream (nếu có)
git remote add upstream https://github.com/original-repo/TachFileTo.git
git pull upstream main
```

### 3.2 Cấu Trúc Thư Mục

```
TachFileTo/
├── Cargo.toml           # Workspace root
├── libs/
│   └── iron_python_bridge/  # PyO3 bridge
│       ├── Cargo.toml
│       ├── src/
│       └── python/          # Python extraction code
├── ui/
│   ├── src/                 # React frontend
│   └── src-tauri/           # Tauri backend
├── test/
│   └── batch_stress_test/   # Benchmark scripts
└── docs/
    └── tests/               # Test contracts
```

---

## 4. CÀI ĐẶT DEPENDENCIES

### 4.1 Rust Setup

```powershell
# Update Rust toolchain
rustup update stable

# Verify version (cần 1.92+)
rustc --version
# Output: rustc 1.92.x (2024 edition)

# Add target nếu cần cross-compile
rustup target add x86_64-pc-windows-msvc
```

### 4.2 Python Virtual Environment

```powershell
# Tạo venv với Python 3.14t
python3.14t -m venv .venv_nogil

# Activate venv
.\.venv_nogil\Scripts\Activate.ps1

# Verify No-GIL enabled
python -c "import sys; print(sys._is_gil_disabled())"
# Output: True

# Install dependencies với uv (nhanh hơn pip)
uv pip install -e .
uv pip install docling[pypdfium2] pydantic ruff pytest psutil
```

> [!NOTE]
> Dùng `pypdfium2` thay vì `pymupdf` nếu wheel chưa sẵn sàng cho cp314t.

### 4.3 Node.js Dependencies (cho UI)

```powershell
cd ui
npm install
cd ..
```

---

## 5. BUILD QTRÌNH

### 5.1 Build Rust Bridge (iron_python_bridge)

```powershell
# Build library
cd libs/iron_python_bridge
cargo build --release

# Verify build thành công
cargo test
cd ../..
```

### 5.2 Build Tauri Desktop App

```powershell
cd ui

# Development mode (hot reload)
npm run tauri dev

# Production build
npm run tauri build -- --release

cd ..
```

**Output location:**
```
ui/src-tauri/target/release/tachfileto.exe
```

### 5.3 (Optional) Compress với UPX

```powershell
# Download UPX từ https://upx.github.io/
# Compress .exe
upx --best ui/src-tauri/target/release/tachfileto.exe

# Verify size
Get-Item ui/src-tauri/target/release/tachfileto.exe | Select-Object Length
# Target: ~50-70MB
```

---

## 6. BUILD VỚI PYOXIDIZER (Advanced)

> [!WARNING]
> PyOxidizer build phức tạp hơn nhưng tạo single binary embed cả Python interpreter.

### 6.1 Cài Đặt PyOxidizer

```powershell
cargo install pyoxidizer
```

### 6.2 Tạo pyoxidizer.bzl Config

```python
# pyoxidizer.bzl
def make_exe():
    dist = default_python_distribution(
        python_version = "3.14",
        flavor = "standalone_static",
    )
    
    policy = dist.make_python_packaging_policy()
    policy.resources_location = "in-memory"
    
    python_config = dist.make_python_interpreter_config()
    python_config.run_module = "extraction"
    
    exe = dist.to_python_executable(
        name = "tachfileto",
        packaging_policy = policy,
        config = python_config,
    )
    
    exe.add_python_resources(
        exe.pip_install(["docling[pypdfium2]", "pydantic", "polars"])
    )
    
    return exe

def make_install(exe):
    return FileManifest.from_python_executable(
        exe,
        include_debug_info = False,
    )

register_target("exe", make_exe)
register_target("install", make_install, depends = ["exe"], default = True)
resolve_targets()
```

### 6.3 Build với PyOxidizer

```powershell
# Build
pyoxidizer build --release

# Output
ls build/release/install/tachfileto.exe

# Compress
upx --best build/release/install/tachfileto.exe
```

---

## 7. TEST BUILD

### 7.1 Quick Smoke Test

```powershell
# Chạy .exe
.\tachfileto.exe

# Test: Drag & drop 1 file PDF
# Expected: Dashboard hiển thị extraction result
```

### 7.2 Batch Stress Test

```powershell
# Activate venv
.\.venv_nogil\Scripts\Activate.ps1

# Run benchmark
python test/batch_stress_test/benchmark.py

# Expected output:
# ============================================================
# 🛡️  ELITE 9 BATCH STRESS TEST
# 📂 Target: E:\...\test\pdf
# 📊 Files: X
# 🔄 Iterations: 30
# ============================================================
# ...
# 📈 Throughput: ~29 items/sec
# ⚡ NO-GIL VERIFIED: Multi-core scaling detected.
```

### 7.3 TDD Test Suite

```powershell
# Run pytest
pytest libs/iron_python_bridge/python/ -v

# Expected: All tests pass
# - test_extraction.py
# - tests/test_extraction_v2.py
```

---

## 8. CI/CD INTEGRATION

### 8.1 GitHub Actions Workflow

```yaml
# .github/workflows/build.yml
name: Build TachFileTo

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: dtolnay/rust-action@stable
        
      - name: Setup Python 3.14t
        uses: actions/setup-python@v5
        with:
          python-version: '3.14t'
          
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          
      - name: Install dependencies
        run: |
          pip install uv
          uv pip install -e .
          cd ui && npm install
          
      - name: Build
        run: |
          cd ui
          npm run tauri build -- --release
          
      - name: Test
        run: |
          cargo test --workspace
          pytest libs/iron_python_bridge/python/ -v
          
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: tachfileto-windows
          path: ui/src-tauri/target/release/tachfileto.exe
```

### 8.2 Required Gates

| Gate | Trigger | Action on Fail |
|:-----|:--------|:---------------|
| `cargo fmt --check` | Every PR | Block merge |
| `cargo clippy` | Every PR | Block merge |
| `cargo test` | Every PR | Block merge |
| `pytest` | Every PR | Block merge |
| `benchmark.py` | Every PR | Warn if regression |

---

## 9. TROUBLESHOOTING

### 9.1 Lỗi "Python not found"

```powershell
# Check PATH
$env:Path -split ';' | Where-Object { $_ -like '*Python*' }

# Set explicit path
$env:PYO3_PYTHON = "C:\Users\<user>\AppData\Local\Programs\Python\Python314t\python.exe"
```

### 9.2 Lỗi "pyo3-ffi ABI mismatch"

```powershell
# Rebuild với correct Python
cargo clean
$env:PYO3_PYTHON = "path/to/python3.14t.exe"
cargo build --release
```

### 9.3 Lỗi "Tauri build failed"

```powershell
# Clear cache và rebuild
cd ui
rm -rf node_modules
npm install
npm run tauri build -- --release
```

### 9.4 Size .exe quá lớn

```powershell
# 1. Enable LTO trong Cargo.toml
[profile.release]
lto = true
codegen-units = 1
strip = true

# 2. Compress với UPX
upx --best tachfileto.exe
```

---

## 10. DISTRIBUTION

### 10.1 Packaging cho End User

```
TachFileTo-v1.0.0-win64/
├── tachfileto.exe       # Main executable
├── README.txt           # Quick start guide
├── LICENSE.txt          # License info
└── sample/
    └── test_qs.pdf      # Sample file for testing
```

### 10.2 Checksum Generation

```powershell
# Generate SHA256
Get-FileHash tachfileto.exe -Algorithm SHA256 | Format-List

# Output to file
Get-FileHash tachfileto.exe -Algorithm SHA256 | 
    Select-Object -ExpandProperty Hash | 
    Out-File -FilePath "tachfileto.exe.sha256"
```

---

## 11. PERFORMANCE TARGETS

| Metric | Target | Hard Limit |
|:-------|:-------|:-----------|
| Cold Start | ≤ 3s | ≤ 5s |
| Throughput | ≥ 25 files/sec | ≥ 20 files/sec |
| Memory | ≤ 500MB peak | ≤ 1GB |
| Binary Size | ≤ 70MB | ≤ 100MB |

> **Đạt được:** 29 files/sec @ ~60MB binary

---

**Author:** TachFileTo Team  
**Last Updated:** 2026-01-25
