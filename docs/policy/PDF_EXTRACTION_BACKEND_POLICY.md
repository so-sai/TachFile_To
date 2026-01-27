# 🛡️ PDF_EXTRACTION_BACKEND_POLICY.md

**Status:** APPROVED – HARD MODE  
**Effective Date:** 2026-01-12  
**Applies To:** Iron Core (Rust Backend)  
**Scope:** PDF / DOCX / Scanned Document Extraction Infrastructure  
**Principle:** Deterministic-first. Performance-gated. AI-last.

---

## §0. TUYÊN NGÔN (MISSION STATEMENT)

PDF Intelligence trong TachFileTo **KHÔNG PHẢI** là tính năng AI.

Nó là **hạ tầng xử lý dữ liệu** – tương đương parser, compiler, query engine.

> **Nếu không đạt hiệu suất gần-native và tính quyết định tuyệt đối → KHÔNG ĐƯỢC PHÉP TỒN TẠI TRONG CORE.**

---

## §1. NGUYÊN TẮC BẤT BIẾN (NON-NEGOTIABLE)

### 1.1 Deterministic Absolute

- Cùng input → cùng output → cùng hash (SHA-256)
- Không phụ thuộc:
  - Network
  - External service
  - Runtime bên ngoài binary

### 1.2 Infrastructure ≠ AI

- **Extraction** = bóc tách sự thật khách quan
- **AI (LLM)** chỉ được phép:
  - Diễn giải
  - Tóm tắt
  - Giải thích kết quả đã được Iron Core tính toán

> [!CAUTION]
> AI **tuyệt đối không được**:
> - Trích xuất số liệu tài chính
> - Quyết định giá, khối lượng, thanh toán

### 1.3 TDD Enforcement

> [!IMPORTANT]
> **Không được viết Production Code trước khi Test Contracts được ký duyệt.**

1. Nghiêm cấm mọi hành vi tạo `Cargo.toml`, `lib.rs`, hoặc bất kỳ code module nào trước khi các Test Contracts sau được Architect ký:
   - `PDF_EXTRACTION_ACCEPTANCE_TEST.md`
   - `PERFORMANCE_GATE_TEST.md`
   - `DETERMINISM_TEST.md`

2. Mọi Unit Test phải chạy được ở chế độ **FAILING** (Red Phase) để chứng minh tính hợp lệ của bài test trước khi viết code để Pass (Green Phase).

3. Bug phát hiện trong production mà không có test case tương ứng → **Vi phạm nghiêm trọng kỷ luật Iron Core**.

---

## §2. CÁC KIẾN TRÚC BỊ CẤM (FORBIDDEN)

### ❌ 2.1 Subprocess-based Integration

Bao gồm nhưng không giới hạn:
- `spawn python`
- stdin/stdout IPC
- JSON-RPC qua process ngoài
- gRPC / HTTP localhost

**Lý do cấm:**
- Latency không kiểm soát
- Debug & audit không quyết định
- Memory + lifecycle ngoài tầm Rust Core

> **Bất kỳ PR nào sử dụng subprocess cho PDF extraction → AUTO-REJECT.**

### ❌ 2.2 Flat RAG / Embedding-first

- Không embed raw PDF
- Không chunk mù
- Không vector search trên toàn corpus

**Lý do cấm:**
- Semantic Collapse ở scale
- Không traceable
- Không audit-proof

### ❌ 2.3 AI-before-Structure

- Không LLM parse PDF trực tiếp
- Không "let the model figure it out"
- AI không được phép chạm vào dữ liệu thô

---

## §3. KIẾN TRÚC DUY NHẤT ĐƯỢC PHÉP (ALLOWED PATH)

### ✅ 3.1 Embedding Python vào Rust (HARD MODE)

```
Iron Core (Rust)
 └── Embedded Python Runtime
       └── Docling (MIT / Apache 2.0)
```

**Yêu cầu bắt buộc:**
- PyO3 + PyOxidizer
- Single binary
- Không dependency runtime bên ngoài

**Rust kiểm soát:**
- Interpreter lifecycle
- Memory
- Threading
- Shutdown

### ✅ 3.2 Distribution Model

- Desktop / CLI / Tauri app
- Offline-first
- Không cloud dependency
- Có thể ký hash + checksum

---

## §4. PERFORMANCE GATE (CỬA SINH TỬ)

### 4.1 Benchmark bắt buộc

| Case | Giới hạn |
|:-----|:---------|
| Cold start (init runtime) | ≤ 5s |
| Warm parse (50–100 pages) | ≤ 1s |
| Memory overhead | ≤ 2× input size |
| Output variance | 0% (bit-identical) |

### 4.2 Quy tắc loại bỏ

- ❌ `> 3s` (warm) → **REJECT**
- ❌ Non-deterministic output → **REJECT**
- ❌ Không thể freeze version → **REJECT**

> **Không có "tạm dùng". Không có "để sau tối ưu".**

---

## §5. OUTPUT CONTRACT (IRON CORE)

### 5.1 Canonical JSON

- Schema versioned
- Field typed
- Không text mơ hồ

**Ví dụ:**

```json
{
  "document_type": "construction_contract",
  "project": "ONSEN_A",
  "sections": [
    {
      "code": "STEEL_01",
      "material": "Thép D10",
      "quantity": 1200,
      "unit": "kg",
      "unit_price": 18000
    }
  ]
}
```

### 5.2 Không được phép

- Không free-text làm nguồn sự thật
- Không để AI suy luận số

---

## §6. LICENSE & COMMERCIAL SAFETY

### 6.1 Allowed

- MIT / Apache 2.0 dependencies
- Closed-source Iron Core
- Dual-license wrapper

### 6.2 Required

- **NOTICE file:**
  > "Portions copyright IBM / Docling project"
- Không dùng trademark "Docling" trong product name

---

## §7. GOVERNANCE & REVIEW

Mọi thay đổi pipeline extraction:
- Phải có benchmark report
- Phải có hash diff test

**Founder / Architect có quyền:**
- Kill feature ngay lập tức nếu vi phạm policy

---

## §8. FINAL CLAUSE (ĐIỀU KHOẢN CUỐI)

> **Iron Core thà KHÔNG CÓ PDF Intelligence còn hơn có một hệ thống chậm, mơ hồ, và không kiểm soát.**

TachFileTo không bán AI.  
TachFileTo bán **kết quả đúng, nhanh, và không thể cãi**.
