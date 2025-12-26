# DEV GUIDE - ĐỌC TRƯỚC KHI CODE

## 🚫 TUYỆT ĐỐI KHÔNG
1. Không đọc các file trong thư mục `docs/archive`. Đó là quá khứ.
2. Không tự ý gửi toàn bộ dữ liệu qua IPC (phải dùng Windowing).
3. Không sửa UI theo cảm tính (phải tuân thủ `UI_DESIGN_SYSTEM.md`).

## ✅ TÀI LIỆU CHÍNH CHỦ (ACTIVE SPECS)
1. **Kiến trúc tổng thể:** [ARCHITECTURE_MASTER.md](file:///e:/DEV/TachFile_To/docs/specs/ARCHITECTURE_MASTER.md)
2. **Giao thức Backend-Frontend:** [IPC_PROTOCOL.md](file:///e:/DEV/TachFile_To/docs/specs/IPC_PROTOCOL.md)
3. **Quy tắc hiển thị:** [UI_DESIGN_SYSTEM.md](file:///e:/DEV/TachFile_To/docs/specs/UI_DESIGN_SYSTEM.md)

## 🚀 TRẠNG THÁI HIỆN TẠI (V2.0)
- Backend: Rust (Calamine Engine) - Giữ data, chia nhỏ gửi đi.
- Frontend: React (Virtualizer) - Chỉ render vùng nhìn thấy.
- Mục tiêu: Xử lý 1.000.000 dòng không Crash.
