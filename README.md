# 📑 TachFileTo-VN 
**Deterministic Validation Engine for Vietnamese Construction Projects**

A tool that extracts truth from chaos — not opinions.

> **🏛️ MDS Reference:** [MDS-ELITE9VN-2026-v1.1](../GOVERNANCE/MDS.md)  
> **📍 Ecosystem Role:** `OPTIONAL` – Heavy Ingestion Tool  
> **⚠️ Classification:** Không tham gia critical payment path (LAW-12)
> **📖 Root Docs:** [README](../README.md)

## 🎯 Mục Đích Cốt Lõi (The Core Purpose)
TachFileTo là hệ thống tự động hóa việc **kiểm tra khối lượng** trong xây dựng. Nó giải quyết một vấn đề cụ thể:

> **"Làm sao để nhanh chóng đối chiếu bảng khối lượng từ file PDF/Excel với thực tế, trước khi thanh toán?"**

## 🧩 Vấn Đề Nó Giải Quyết (Cụ Thể)
**1. Xử lý Dữ Liệu Thô Từ Hiện Trường**
*   **OCR & Trích xuất bảng từ PDF:** Tự động đọc file scan (kể cả file lớn >50MB), bản vẽ, hồ sơ chất lượng có bảng biểu.
*   **Xử lý đa dạng định dạng:** Đọc file Excel hiện có, chuẩn hóa về một cấu trúc duy nhất.
*   **Sửa lỗi font chữ Việt Nam:** Tự động phát hiện và chuyển đổi font TCVN3, VNI về Unicode.

**2. Kiểm Tra Tính Hợp Lý & Cảnh Báo Rủi Ro**
*   **Phát hiện sai lệch:** So sánh khối lượng giữa các giai đoạn, phát hiện chênh lệch bất thường.
*   **Áp dụng quy tắc nghiệp vụ Việt Nam:** Tính toán lại theo đơn giá, định mức, kiểm tra làm tròn số.
*   **Gắn bằng chứng trực quan:** Liên kết từng dòng dữ liệu với hình ảnh "evidence" được crop chính xác từ bản vẽ gốc.

**3. Trình Bày Cho Người Ra Quyết Định**
*   **Giao diện Founder-first:** Từ bảng dữ liệu chi tiết (QS) tổng hợp thành tín hiệu rõ ràng: **An toàn / Cảnh báo / Nguy cơ**.
*   **Truy xuất nguồn gốc trong 1 cú click:** Từ tín hiệu cảnh báo có thể drill-down ngay xuống dòng dữ liệu gốc và hình ảnh bằng chứng.

## 🧠 Nguyên Tắc Thiết Kế Sắt Đá (Iron Core)
1.  **Xác Định Trên Thông Minh (Determinism over Intelligence):** Cùng một đầu vào → luôn cho cùng một kết quả. Không có AI "phán đoán mù".
2.  **Giao Diện Không Tính Toán (UI Never Thinks):** Mọi logic nghiệp vụ nằm trong [`iron_coreVN`](../iron_core/README.md). Frontend chỉ hiển thị.
3.  **Ưu Tiên Hiệu Năng (Performance is a Feature):** Xử lý file lớn (>50MB) là chuyện bình thường. Không có spinner giả dối.
4.  **Tôn Trọng Thực Tế Việt Nam (Vietnamese Reality First):** Thuật ngữ, cách tính toán, quy chuẩn xây dựng Việt Nam là ưu tiên hàng đầu.

## 🏗️ Nguyên Tắc Kiến Trúc
*   **Core Deterministic:** Logic duy nhất được viết bằng Rust, đảm bảo tính xác định.
*   **Desktop-First, Offline-First:** Ứng dụng chạy độc lập trên Windows, ưu tiên tốc độ và quyền riêng tư.
*   **Contracts Rõ Ràng:** Giao tiếp giữa các module thông qua các data contract được định nghĩa chặt chẽ.

## 📖 Bản Đồ Tài Liệu

### 📂 Cấu trúc dự án (IIP v1.1)
- **Triết lý cốt lõi:** `.project-context/ANTI_GRAVITY.md` - Constitutional principles
- **Quy tắc Agent:** `.project-context/AGENT_RULES.md` - AI agent guidelines
- **Quản lý Mission:** `.project-context/MISSION_CONTROL.json` - State tracking
- **Nhiệm vụ hiện tại:** `.project-context/ACTIVE_MISSION.md` - Current work
- **Bài học kinh nghiệm:** `LESSONS.md` - Anti-patterns và lỗi đã sửa

### 📋 Specifications & Guides
- **Hướng dẫn tổng quan:** `docs/GUIDE.md` - Single source of truth
- **Trạng thái hệ thống:** `docs/specs/MASTER_V3.0_DASHBOARD.md`
- **Quy tắc nghiệp vụ:** `docs/specs/RULE_PACK_NHA_NUOC_V1.1.md`
- **Giao thức IPC:** `docs/specs/IPC_PROTOCOL.md`
- **Hệ thống thiết kế UI:** `docs/specs/UI_DESIGN_SYSTEM.md`
- **Hợp đồng dữ liệu:** `docs/specs/TRUTH_CONTRACT_V1.md`
- **Ranh giới hệ sinh thái:** `docs/BOUNDARY_MANIFEST.md`
- **Lịch sử thay đổi:** `docs/CHANGELOG.md`

### 🗃️ Lưu trữ
- **Tài liệu cũ:** `.project-context/ARCHIVE/` - Legacy specs và reports

---

## ⚠️ Giới Hạn Trách Nhiệm

TachFileTo **không thay thế kỹ sư**, không tự động phê duyệt thanh toán — nó chỉ phơi bày sự thật để con người chịu trách nhiệm.

---

> **Triết lý cuối cùng:** "Bảng dashboard không phải để ngắm. Nó tồn tại để ra quyết định. Nếu TachFileTo chuyển màu đỏ, ai đó phải dừng lại và hành động."
