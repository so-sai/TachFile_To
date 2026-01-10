## AGENT RULES (IIP v1.1)

1. **Zero-Assumption (Không suy diễn):** 
   - Chỉ code dựa trên file nhìn thấy. Nếu `docs/specs` chưa rõ, phải hỏi lại Owner.
   
2. **Scope Containment (Khoanh vùng):**
   - Chỉ sửa các file được liệt kê trong `ACTIVE_MISSION.md`.
   - Cấm tự ý sửa file cấu hình `Cargo.toml` trừ khi nhiệm vụ yêu cầu.

3. **Rust Safety Protocol:**
   - Vì Rust biên dịch lâu: Hãy suy nghĩ kỹ Logic trước khi viết code.
   - Ưu tiên dùng `cargo check` để kiểm tra cú pháp nhanh thay vì `cargo build`.
   - Không được `unwrap()` bừa bãi, hãy xử lý lỗi bằng `Result/Option`.

4. **Evidence First:**
   - Luôn chụp ảnh màn hình Terminal kết quả test.

5. **Mission State Machine (IIP v1.1):**
   - Mọi công việc phải được định nghĩa trong `ACTIVE_MISSION.md`.
   - Trạng thái mission được quản lý qua `MISSION_CONTROL.json`.
   - Không được nhảy bước: PLANNING → AUDITING → EXECUTING → TESTING → DONE.

6. **Skeptic Protocol:**
   - Mọi mission phải qua AGENT S (Skeptic) trước khi EXECUTING.
   - Skeptic có quyền phủ quyết (FAIL verdict).
   - Human Architect có quyền cuối cùng.

7. **Constitutional Compliance:**
   - Tất cả code phải tuân thủ `ANTI_GRAVITY.md`.
   - Vi phạm nguyên tắc = vi phạm kiến trúc.
