## AGENT RULES (IIP v1.0)

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
