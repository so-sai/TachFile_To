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

## 🤖 WORKING WITH AI AGENTS (IIP v1.1)

### Governance Structure:
1. **Read Philosophy:** Start with `.project-context/ANTI_GRAVITY.md`
2. **Check Mission Control:** Review `.project-context/MISSION_CONTROL.json`
3. **Follow Agent Rules:** Adhere to `.project-context/AGENT_RULES.md`

### Mission Workflow (State Machine):
```
PLANNING → AUDITING → EXECUTING → TESTING → DONE
```

1. **PLANNING:** Create `ACTIVE_MISSION.md` with clear scope
2. **AUDITING:** Skeptic (AGENT S) reviews for boundary violations
3. **EXECUTING:** Implement after Human Architect approval
4. **TESTING:** Verify with `cargo test` and manual validation
5. **DONE:** Archive mission report

### Example Mission:
See `.project-context/ACTIVE_MISSION.md` for current mission structure.
