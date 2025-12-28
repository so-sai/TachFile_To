# DEV GUIDE - ĐỌC TRƯỚC KHI CODE

## 🚫 TUYỆT ĐỐI KHÔNG
1. Không đọc các file trong thư mục `.project-context/ARCHIVE/`. Đó là quá khứ.
2. Không tự ý gửi toàn bộ dữ liệu qua IPC (phải dùng Windowing).
3. Không sửa UI theo cảm tính (phải tuân thủ `UI_DESIGN_SYSTEM.md`).

## ✅ TÀI LIỆU CHÍNH CHỦ (ACTIVE SPECS)
1. **Kiến trúc tổng thể:** [ARCHITECTURE_MASTER.md](file:///e:/DEV/TachFile_To/docs/specs/ARCHITECTURE_MASTER.md)
2. **Giao thức Backend-Frontend:** [IPC_PROTOCOL.md](file:///e:/DEV/TachFile_To/docs/specs/IPC_PROTOCOL.md)
3. **Quy tắc hiển thị:** [UI_DESIGN_SYSTEM.md](file:///e:/DEV/TachFile_To/docs/specs/UI_DESIGN_SYSTEM.md)

## 🚀 TRẠNG THÁI HIỆN TẠI (V3.0)
- Backend: Rust (Tauri 2.x, Polars 0.52) - Iron Core với Smart Header Detection
- Frontend: React 19 + TypeScript - Brutalist Dark Mode UI
- Architecture: Desktop-first, Offline-only, Deterministic

## 🤖 WORKING WITH AI AGENTS (IIP v1.0)

### Workflow chuẩn:
1. **Đọc hiến pháp:** Agent luôn bắt đầu bằng đọc `.project-context/PROJECT_PROMPT.md`
2. **Tạo mission:** Khi có task mới, cập nhật `.project-context/ACTIVE_MISSION.md` với:
   - Objective rõ ràng
   - Files in scope (chỉ sửa những file được liệt kê)
   - Success criteria
3. **Thực thi:** Agent làm việc theo `AGENT_RULES.md` (Zero-Assumption, Evidence First)
4. **Verify:** Luôn chạy `cargo check` và `cargo test` trước khi commit

### Ví dụ mission:
```markdown
## Mission: Fix Excel Header Detection Bug
**Status:** In Progress
**Started:** 2025-12-28

### Files in Scope
- [ ] `ui/src-tauri/src/excel_engine.rs`
- [ ] `ui/src-tauri/src/normalizer.rs`

### Success Criteria
- [ ] `cargo test` pass
- [ ] Xử lý được file có header ở row 10
```
