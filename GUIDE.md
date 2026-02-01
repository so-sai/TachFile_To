# 🛡️ CẨM NANG GIÁM ĐỊNH TACHFILETO (V1.0.0) ⚖️

Chào mừng anh đến với **TachFileTo V1.0.0**. Đây không chỉ là một phần mềm; đây là **Trạm làm việc Giám định** giúp anh bóc tách "Sự thật kỹ thuật" từ những đống dữ liệu hỗn loạn trong ngành Xây dựng (AEC) tại Việt Nam.

---

## 🏛️ 1. TRIẾT LÝ: TẠI SAO LÀ "PHÁN QUYẾT"?

Khi xuất dữ liệu ra Excel hoặc Markdown, anh sẽ thấy cột `PHAN_QUYET_TACHFILETO`. Đây là linh hồn của hệ thống:

* **SẠCH (CLEAN)**: Mọi chữ số ở hàng này đã được truy vết và xác thực 100%. Anh có thể tin tưởng hoàn toàn.
* **VẤN ĐỀ (SUSPICIOUS)**: Có dấu hiệu nhiễu mã hóa (font) hoặc sai lệch làm tròn. Anh **bắt buộc** phải kiểm tra lại tại Bàn Giám định (Evidence Pane).
* **TỪ CHỐI (REJECTED)**: Dữ liệu không có nguồn gốc trong tài liệu gốc hoặc vi phạm "Hiến pháp Sự thật".

> **Lưu ý**: Với tư cách là Chủ trì kỹ sư hoặc QS, khi anh sử dụng dữ liệu này, anh đang đặt bút ký vào tính toàn vẹn pháp lý của nó.

---

## 🛠️ 2. CHIẾN THUẬT SỬA LỖI: TRỊ DỨT ĐIỂM FONT CHỮ (MOJIBAKE)

Các dự án tại Việt Nam thường là một "ma trận" trộn lẫn giữa Unicode, TCVN3 (ABC) và VNI. TachFileTo sẽ phát hiện chúng, nhưng anh là người đưa ra phán quyết cuối cùng.

* **Bàn Giám định (Evidence Pane)**: Khi một ô dữ liệu bị đánh dấu đỏ, hãy nhìn sang phải. Hệ thống sẽ hiển thị ảnh cắt (crop) trực tiếp từ file PDF gốc.
* **Giải mã kép**: Hệ thống sẽ gợi ý cả hai phiên bản (TCVN3 và VNI). Anh chỉ cần chọn phiên bản khớp với hình ảnh thực tế đang nhìn thấy.
* **Chế độ Niêm phong**: Một khi anh đã xác nhận sửa lỗi, hệ thống sẽ đóng dấu băm SHA-256. Ô dữ liệu đó chính thức trở thành **Sự thật Hợp lệ**.

---

## ⚡ 3. MẸO VẬN HÀNH & HIỆU SUẤT

* **Nạp dữ liệu hàng loạt**: Anh có thể kéo-thả cả một thư mục chứa hàng trăm file PDF/Excel. Hệ thống sẽ xử lý song song để tiết kiệm thời gian cho anh.
* **Truy vết 1-Click**: Click vào bất kỳ con số "Mâu thuẫn" nào trên Dashboard để nhảy ngay đến vị trí của nó trên bản vẽ PDF gốc.
* **An toàn Windows**: Nếu anh gặp lỗi "Permission Denied" khi xuất file, đơn giản là hãy đóng file Excel đó lại. TachFileTo bảo vệ dữ liệu của anh khỏi bị hỏng do ghi đè trùng lặp.

---

## 📉 4. XUẤT BẢN HỒ SƠ GIÁM ĐỊNH

* **Markdown (.md)**: Đây là "Hồ sơ Pháp lý". Hãy dùng nó để lưu trữ, đẩy lên Git hoặc đính kèm vào báo cáo kiểm toán chính thức.
* **Excel (.xlsx)**: Đây là "Công cụ Thực chiến". File được định dạng chuẩn với tiêu đề cố định (frozen headers) và định dạng số thực (`f64`), sẵn sàng cho các phép tính tiếp theo của anh.

---

> *TachFileTo: Trích xuất Sự thật từ đống Dữ liệu hỗn loạn.*
> © 2026 Hệ sinh thái Xây dựng DT.
