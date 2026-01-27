# Elite PDF 🚀

**High-Performance No-GIL MuPDF Wrapper for Python 3.14+**

Elite PDF is a Rust-based PDF processing library designed from the ground up for Python's free-threaded (No-GIL) mode. It provides thread-safe access to MuPDF's powerful PDF rendering and text extraction capabilities.

## Features

- ✅ **No-GIL Native**: Designed for Python 3.14+ free-threaded mode
- ✅ **Thread-Safe**: Per-thread MuPDF context isolation
- ✅ **Zero-Copy**: Efficient data handling where possible
- ✅ **Parallel Processing**: Rayon-powered multi-threaded operations
- ✅ **High Performance**: Rust + MuPDF = maximum speed

## Quick Start

```python
from elite_pdf import EliteDocument

# Open a PDF
doc = EliteDocument("example.pdf")

# Extract text from a page
text = doc.extract_text(0)

# Render a page to PNG
png_bytes = doc.render_page(0, dpi=150)

# Extract all pages in parallel
all_text = doc.extract_all_text()
```

## Building

### Prerequisites

- Rust 1.92+ (Edition 2024)
- Python 3.14+ (free-threaded build recommended)
- MuPDF development libraries
- maturin (`pip install maturin`)

### Build for Development

```bash
cd libs/elite_pdf
maturin develop --release
```

### Build for Release (cp314t wheel)

```bash
maturin build --release --interpreter python3.14t
```

## Architecture

```
elite_pdf/
├── Cargo.toml          # Rust package configuration
├── build.rs            # MuPDF linking configuration
├── pyproject.toml      # Python package configuration
├── src/
│   └── lib.rs          # Main library code
└── python/             # Python stubs (optional)
```

## Status

🚧 **Alpha** - Core structure implemented, MuPDF bindings in progress.

## License

MIT OR Apache-2.0 (dual-licensed)

---

*Part of the Elite 9 TachFileTo ecosystem*
