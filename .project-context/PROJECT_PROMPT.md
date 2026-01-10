# PROJECT CONTEXT: app-tool-TachFileTo

**IIP Protocol:** v1.1  
**Project Type:** APP  
**Owner:** so-sai

---

## 0. GOVERNANCE (IIP v1.1)

**Core Philosophy:** See `ANTI_GRAVITY.md`  
**Mission Control:** See `MISSION_CONTROL.json`  
**Agent Rules:** See `AGENT_RULES.md`

**State Machine:** PLANNING → AUDITING → EXECUTING → TESTING → DONE

---

## 1. IDENTITY (ĐỊNH DANH)
- **Mục tiêu:** Ứng dụng Desktop (Tauri) xử lý/tách file dữ liệu lớn, đảm bảo tính tất định (Deterministic Validation Engine) cho dự án Xây dựng.
- **Tech Stack:**
  - **Core:** Rust (Tauri 2.x), Polars 0.52 (Dataframe), Calamine 0.32 (Excel).
  - **UI:** React 19, TypeScript, Vite.
  - **Architecture:** Desktop-first, Offline-only.
- **Owner:** so-sai

## 2. KNOWLEDGE BASE (KHO TRI THỨC - KẾT NỐI)
Agent bắt buộc phải tham chiếu các tài liệu sau trong `docs/specs/` trước khi thực hiện logic tương ứng:
1. **Giao tiếp Frontend-Backend:** Đọc `docs/specs/IPC_PROTOCOL.md` (Command/Payload).
2. **Quy tắc Nghiệp vụ:** Đọc `docs/specs/RULE_PACK_NHA_NUOC_V1.1.md`.
3. **Cấu trúc Dữ liệu:** Đọc `docs/specs/TRUTH_CONTRACT_V1.md`.
4. **Giao diện:** Đọc `docs/specs/UI_DESIGN_SYSTEM.md`.
5. **Dashboard:** Đọc `docs/specs/MASTER_V3.0_DASHBOARD.md`.

## 3. BOUNDARIES (BIÊN GIỚI)
- **Ecosystem Role:** Data Ingestion Organ (xem `docs/BOUNDARY_MANIFEST.md`)
- **Phạm vi:** Chỉ xử lý logic trong `src-tauri` (Rust) và `src` (React).
- **Integration:** Giao tiếp với iron_core qua `docs/specs/INGESTION_SCHEMA.json`
- **Cấm:** 
  - Tuyệt đối không upload dữ liệu lên Cloud.
  - Không sửa các file trong `docs/specs/archive/`.
  - **KHÔNG được tạo business logic hoặc ghi vào iron_core database.**
  - **KHÔNG được lưu project metadata trong UI preferences.**
- **An toàn:** Luôn chạy `cargo check` trước khi confirm code Rust.

## 4. DEFINITION OF DONE
- [ ] Rust: `cargo test` pass.
- [ ] React: Không còn lỗi TypeScript.
- [ ] Logic khớp với `docs/specs/`.
