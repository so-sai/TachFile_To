# TachFileTo

**Engine Kiểm Tra Dữ Liệu Hồ Sơ Xây Dựng Việt Nam (Deterministic File Validation Engine)**

*Công cụ trích xuất sự thật từ sự hỗn loạn dữ liệu — không phán đoán, không phê duyệt.*

---

### 🚀 Vì sao TachFileTo tồn tại

TachFileTo sinh ra từ thực tế rằng các phần mềm hiện có thường sụp đổ khi xử lý hồ sơ xây dựng lớn, nặng, nhiều file scan và tồn tại lâu dài. Chúng chậm, mù dữ liệu, chuyển đổi không bền, sinh file rác và không có trí nhớ hệ thống giữa các lần xử lý. 

TachFileTo được tạo ra để giải quyết **hạ tầng xử lý dữ liệu**, không phải để phán đoán nghiệp vụ.

---

## 🎯 Mục Đích Cốt Lõi

TachFileTo là một **engine xử lý & kiểm tra dữ liệu hồ sơ xây dựng** ở mức **kỹ thuật thuần túy**.

Nó trả lời **một câu hỏi duy nhất**:

> *“Dữ liệu trong các file PDF / Excel này thực sự nói gì, và có nhất quán với nhau hay không?”*

TachFileTo **không ra quyết định**, **không phê duyệt**, **không thay thế con người**.

---

## 🧩 Phạm Vi Vấn Đề Giải Quyết

### 1. Xử lý dữ liệu thô từ hồ sơ hiện trường

* Trích xuất bảng từ PDF (kể cả file scan, file lớn >50MB).
* Đọc & chuẩn hóa Excel về **một cấu trúc dữ liệu thống nhất**.
* Tự động phát hiện và chuyển đổi font tiếng Việt (TCVN3, VNI → Unicode).
* Chuẩn hóa tiêu đề, cột, đơn vị theo ngữ cảnh hồ sơ xây dựng Việt Nam.

👉 **Đầu ra:** dữ liệu sạch, có cấu trúc, có thể kiểm chứng.

---

### 2. Kiểm tra tính nhất quán & phát hiện sai lệch dữ liệu

* So sánh khối lượng giữa các file / giai đoạn.
* Phát hiện chênh lệch bất thường, thiếu dòng, trùng dòng, sai làm tròn.
* Áp dụng **quy tắc tính toán xác định (deterministic rules)** — không suy đoán.
* Gắn bằng chứng trực quan: mỗi dòng dữ liệu có thể truy ngược về **ảnh crop chính xác từ tài liệu gốc**.

👉 **Đầu ra:** danh sách *facts & discrepancies*, không phải kết luận nghiệp vụ.

---

### 3. Trình bày dữ liệu cho người chịu trách nhiệm

* Giao diện **Ưu tiên Người điều hành (Founder/Engineer-first)**: ưu tiên quan sát sự thật, không diễn giải.
* Từ bảng dữ liệu chi tiết → tín hiệu kỹ thuật:
  **Khớp (Consistent) / Sai lệch (Inconsistent) / Cần kiểm tra (Requires Review)**
* Truy xuất 1-click: từ dữ liệu → file gốc → hình ảnh bằng chứng.

👉 **Giao diện chỉ hiển thị — không “suy nghĩ” thay người dùng.**

---

## 🧠 Nguyên Tắc Thiết Kế (Iron Core)

* **Xác định trên Thông minh (Determinism over Intelligence)**
  Cùng đầu vào → luôn cùng đầu ra. Không AI phán đoán mơ hồ.

* **Giao diện không tính toán (UI Never Thinks)**
  Mọi logic nằm trong Core (Rust). Frontend không chứa nghiệp vụ.

* **Hiệu năng là tính năng (Performance is a Feature)**
  File lớn, nhiều trang, nhiều bảng là trạng thái bình thường. Không spinner giả dối.

* **Thực tế Việt Nam là số 1 (Vietnamese Reality First)**
  Thuật ngữ, cách trình bày, đặc thù hồ sơ Việt Nam là ưu tiên — *nhưng không thay thế chuẩn mực kỹ thuật*.

---

## 🏗️ Nguyên Tắc Kiến Trúc

* **Core Deterministic (Rust)**
  Một nguồn sự thật duy nhất cho logic xử lý.

* **Desktop-first, Offline-first**
  Chạy độc lập trên Windows. Ưu tiên tốc độ và quyền riêng tư.

* **Hợp đồng dữ liệu chặt chẽ (Strict Data Contracts)**
  Mọi module giao tiếp qua contract rõ ràng, có phiên bản.

---

## 📖 Bản Đồ Tài Liệu

### 📂 Cấu trúc dự án (IIP v1.0)

* Hiến pháp dự án: `.project-context/PROJECT_PROMPT.md`
* Quy tắc AI: `.project-context/AGENT_RULES.md`
* Nhiệm vụ hiện tại: `.project-context/ACTIVE_MISSION.md`
* Bài học kinh nghiệm: `LESSONS.md`
* Tài liệu lưu trữ: `.project-context/ARCHIVE/`

### 📋 Thông số & Hướng dẫn (Specifications)

* Trạng thái hệ thống: `docs/specs/MASTER_V3.0_DASHBOARD.md`
* Quy tắc kiểm tra dữ liệu: `docs/specs/RULE_PACK_CORE_V1.0.md`
* Giao thức IPC: `docs/specs/IPC_PROTOCOL.md`
* Hợp đồng dữ liệu: `docs/specs/TRUTH_CONTRACT_V1.md`
* Hệ thống thiết kế UI: `docs/specs/UI_DESIGN_SYSTEM.md`
* Hướng dẫn sử dụng: `docs/GUIDE.md`
* Lịch sử thay đổi: `docs/CHANGELOG.md`

---

## ⚠️ Giới Hạn Trách Nhiệm (Boundary)

TachFileTo:

* ❌ Không phê duyệt thanh toán
* ❌ Không đưa ra kết luận pháp lý hay nghiệp vụ
* ❌ Không thay thế kỹ sư, QS, hay chủ đầu tư

✅ Nó **chỉ làm một việc**:
**Phơi bày dữ liệu và sự nhất quán của chúng một cách chính xác và có thể kiểm chứng.**
