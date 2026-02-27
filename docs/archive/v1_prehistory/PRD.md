# TACHFILETO CORE+ (V1.1 PRD)

**Mission:**  
Convert large PDF/DOCX (>50MB), including scanned documents, into clean AI-ready Markdown, 100% offline, without UI lag. Optionally compare two documents for numerical and structural differences.

**Non-Goals (Out of Scope for V1):**  
Legal compliance (Decree 254), auditing, audit trails (SHA-256 chaining), domain-specific validation (R02-R04), formatting preservation (beyond AI readability).

**Success Criteria:**
- User can drag a 100MB scanned PDF and get clean Markdown in under 90 seconds (CPU-only).
- Zero UI freeze during processing.
- Generated Markdown preserves logical hierarchy and removes pagination artifacts.

---

## 1. CORE FEATURES (V1.0)

### A. Input Pipeline
- **Formats:** PDF (text + scan) > 50MB, DOCX > 50MB.
- **Handling:** Streaming processing to maintain < 2GB RAM usage.

### B. Processing Engine (The "Cleaner")
- **OCR:** CPU-only OCR via worker threads (e.g., using Tesseract/Docling via PyO3).
- **Normalization:** 
  - Standardize Unicode (TCVN3, VNI -> Unicode).
  - Strip redundant headers, footers, and page numbers.
  - Consolidate broken paragraphs.
- **Structure Preservation:** Maintain H1/H2/Bullet/Table structures in a way that is easily parsed by LLMs.

### C. Output
- Clean Markdown (`.md`).
- Clean DOCX (`.docx`).
- Copy-to-clipboard functionality specifically formatted for AI context windows.

### D. Deterministic Diff (Cross-File Verification)
- **Table-Aware AST Conversion:** Documents are converted to an internal AST (Abstract Syntax Tree) before comparison, never pure text diffing.
- **Table Diff:** Detect added/removed rows or changed values between two files.
- **Heading Diff:** Verify structural changes in document outlines.
- **Delta Detection:** Instantly highlight numerical discrepancies between versions (e.g., BOQ revisions).

---

## 2. ARCHITECTURE OVERVIEW

- **Frontend:** Svelte 5 + Vite + Tailwind v4. Ultra-minimalist UI focusing on parallel ingestion (Dropzone A & B).
- **Backend:** Tauri 2 (Rust).
- **Processing:** Asynchronous Rust workers pushing heavy lifting (OCR, Parsing) off the main thread.
- **Diff Engine:** Custom logic comparing normalized AST representations.

*No executioner complexity, no resource court, no Polars (unless strictly necessary for table diffing).*
