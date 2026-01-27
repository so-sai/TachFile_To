# 🚩 ACTIVE MISSION #002: Hard-Mode Skeleton Setup

**Status:** IN PROGRESS  
**Started:** 2026-01-12  
**Owner:** Architect

---

## 1. Mục tiêu (Objectives)

- [x] Khởi tạo thư mục `docs/policy/` với `PDF_EXTRACTION_BACKEND_POLICY.md`
- [x] Khởi tạo thư mục `docs/benchmark/` với `BENCHMARK_TEMPLATE.md`
- [x] Khởi tạo `libs/iron_python_bridge/` (Rust library skeleton)
- [x] Thiết lập file `pyoxidizer.bzl` với cấu hình tối giản
- [ ] Cập nhật workspace `Cargo.toml` để include library mới
- [ ] Viết Unit Test: Khởi tạo Embedded Interpreter và trả về version Python

## 2. Files được tạo/sửa

| File | Status | Description |
|:-----|:-------|:------------|
| `docs/policy/PDF_EXTRACTION_BACKEND_POLICY.md` | ✅ Done | Luật sắt 8 điều khoản |
| `docs/benchmark/BENCHMARK_TEMPLATE.md` | ✅ Done | Template đối soát SLA |
| `libs/iron_python_bridge/Cargo.toml` | ✅ Done | Crate config (PyO3 ready) |
| `libs/iron_python_bridge/src/lib.rs` | ✅ Done | Skeleton với tests |
| `pyoxidizer.bzl` | ✅ Done | Single-binary config |
| `Cargo.toml` | 🔄 Pending | Workspace update |

## 3. Universal Check Commands

```bash
# Verify skeleton compiles
cd e:/DEV/elite10-ecosystem/app-tool-TachFileTo
cargo check -p iron_python_bridge

# Run skeleton tests
cargo test -p iron_python_bridge

# (Future) PyOxidizer build
# pyoxidizer build --release
```

## 4. Success Criteria

- [ ] `cargo check -p iron_python_bridge` passes
- [ ] `cargo test -p iron_python_bridge` passes (4 tests)
- [ ] No external Python dependency for skeleton mode

## 5. Next Mission

**Mission #003: Docling Integration PoC**
- Evaluate Docling CPU-only performance
- Run benchmark against `BENCHMARK_TEMPLATE.md`
- Decision: Proceed with PyOxidizer or find alternative

---

> [!IMPORTANT]
> Skeleton mode chỉ dùng để verify architecture. PyO3 thực sự sẽ được enable khi có Docling integration.
