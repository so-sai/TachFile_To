Aligned with: [MDS_ALIGNMENT_V1.0.md]

# UI DESIGN SYSTEM V1.0

## *(Enterprise Visual Grammar – Corrected & Hardened)*

> **Ghi chú phiên bản**
> 
> V1.2 sửa các giả định sai của V1.1 liên quan đến iOS nostalgia, scrollbar, và bảng dữ liệu lớn (≥100k rows).
> Trọng tâm chuyển từ *"thẩm mỹ nhẹ nhàng"* sang *"định vị – hiệu suất – nhận thức"*.

---

## **1. TRIẾT LÝ CỐT LÕI** - *"UI Never Thinks"*

Giao diện chỉ là lớp kính hiển thị sự thật từ Core.

- **UI NEVER THINKS**: Frontend không chứa logic nghiệp vụ. Chỉ hiển thị kết quả từ Rust.
- **FOUNDER / ENGINEER FIRST**: Ưu tiên mật độ dữ liệu và khả năng drill-down (truy xuất nguồn gốc) hơn là vẻ đẹp hình thức.
- **ƯU TIÊN NỘI DUNG**: Giao diện khiêm tốn phục vụ dữ liệu, không cạnh tranh với nó.
- **RÕ RÀNG TỨC THÌ**: Văn bản đọc được ngay, hệ thống phân cấp hiển nhiên.
- **ĐỘ SÂU TỐI GIẢN**: Chỉ dùng đường viền và đổ bóng nhẹ để phân tầng.
- **KHÔNG GÂY XAO NHÃNG**: Không gradient, không trong suốt, không bo góc nặng ở khu vực dữ liệu.

---

## **2. CẤU TRÚC BỐ CỤC** - *Layout "Thánh Địa" Tinh Chế*

### **Ứng dụng gốc**
```css
h-screen w-screen overflow-hidden flex bg-gray-100
/* Nền xám ấm hơn gray-50 để giảm chói mắt */
```

### **Thanh bên cố định**
```css
w-64 bg-white border-r-2 border-gray-300
/* Viền phải dày 2px - phân cách mạnh mẽ */
```

### **Khu vực chính**
```css
flex-1 flex flex-col bg-white
/* Nền trắng tinh khiết cho khu vực dữ liệu */
```
- **Đầu trang**: `h-14 border-b-2 border-gray-300 px-6`
- **Thân trang**: `flex-1 overflow-auto p-0`
  *Không padding - bảng chiếm toàn bộ chiều rộng*

---

## **3. BẢNG MÀU CHUẨN** - *Tương phản cao, phong cách iOS 10*

### **Màu nền**
- `bg-white` (khu vực dữ liệu)
- `bg-gray-100` (khung ứng dụng)

### **Màu viền**
- `border-gray-300` (viền đậm, rõ ràng - không dùng gray-200 quá nhạt)

### **Màu chữ**
| Loại | Mã màu | Sử dụng |
|------|--------|---------|
| **Chính** | `text-gray-900` | Văn bản quan trọng |
| **Phụ** | `text-gray-600` | Văn bản hỗ trợ |
| **Nhạt** | `text-gray-500` | Nhãn, metadata |

### **Màu nhấn & Trạng thái**
- **Hành động chính**: `blue-700` (xanh đậm chuyên nghiệp)
- **Thành công**: `green-700` (xanh lá đậm)
- **Lỗi**: `red-700` (đỏ đậm)

---

## **4. HỆ THỐNG CHỮ** - *SF Pro / Phông hệ thống*

### **Phông chữ**
```css
font-sans
/* Tự động: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto */
```

### **Thang đo chữ (CỨNG NHẮC)**
| Vai trò | Thuộc tính | Kích thước |
|---------|-----------|------------|
| **Tiêu đề trang** | `text-2xl font-semibold` | 24px |
| **Tiêu đề mục** | `text-lg font-medium` | 18px |
| **Đầu bảng** | `text-xs font-semibold uppercase tracking-wider text-gray-600` | 12px |
| **Ô bảng** | `text-sm font-normal text-gray-900` | 14px |
| **Nhãn nhỏ** | `text-xs text-gray-500` | 12px |

### **➕ BỔ SUNG V1.2 – SỐ LIỆU (QUAN TRỌNG)**

```css
.tabular-nums {
  font-variant-numeric: tabular-nums;
}
```

> **Lý do:**
> 
> Scan dọc số liệu trong bảng lớn **bắt buộc** phải có `tabular-nums`.
> Không có → mắt phải "canh" từng chữ số → tăng cognitive load.

---

## **5. THIẾT KẾ BẢNG** - *VirtualLedger (Giống Excel)*

### **Nguyên tắc tổng thể**
- **Không card ngoài**, không đổ bóng, không bo góc
- **Bảng chiếm toàn bộ** khu vực chính

### **Quy tắc viền**
- **Tất cả ô**: `border border-gray-300`
- **Hàng tiêu đề**: `border-b-2 border-gray-300`

### **Trạng thái hàng**
- **Di chuột**: `hover:bg-gray-50` (xám nhạt, không xanh)
- **Sọc zebra** (tuỳ chọn): `even:bg-gray-50`

### **⚠️ CHỈNH SỬA V1.2 – ENTERPRISE DATA🚨 **TÀI LIỆU ĐÃ LỖI THỜI - KHÔNG SỬ DỤNG**
**Version thực tế:** V2.3 (Perception Engine + Polars 0.52)
**Cập nhật cuối:** 2025-12-26
**Trạng thái:** ARCHIVED - Chỉ để tham khảo lịch sử
→ Xem [ARCHITECTURE_V2.3.md](file:///e:/DEV/TachFile_To/docs/specs/ARCHITECTURE_V2.3.md) để biết source of truth

# UI DESIGN SYSTEM v1.0

**❌ LOẠI BỎ (V1.1 – sai ngữ cảnh):**
```css
px-4 py-3   /* Padding quá lớn cho bảng 100k rows */
```

**✅ THAY THẾ (V1.2 – Enterprise Density):**
```css
/* Row chuẩn Enterprise */
height: 32px;
padding: 0 12px;
```

> **Giải thích:**
> 
> - `32px` = chuẩn Excel / Bloomberg
> - `py-3` (≈24px) phá Data Density
> - Padding lớn + virtualization = lãng phí viewport

### **Tiêu đề cố định**
- **Cố định trên cùng**: `sticky top-0`
- **Viền dưới**: `border-b-2`

---

## **6. ĐIỀU KHIỂN & KHOẢNG CÁCH** - *Hệ lưới 8px*

### **Đơn vị cơ sở**
- **8px** = `2` trong Tailwind
- **Mọi spacing** là bội số của 8px: `p-4`, `gap-4`, `space-y-6`

### **Kích thước chuẩn**
- **Nút bấm**: Chiều cao tối thiểu `h-10` (40px ≈ 44pt iOS)
- **Ô nhập**: `h-9` hoặc `h-10`

### **Quy tắc spacing**
```css
/* ĐÚNG */ p-4, m-6, gap-4, space-y-6
/* SAI */ p-3, m-5, gap-3, space-y-5
```

---

## **7. CẤM TUYỆT ĐỐI** - *Không thương lượng*

### **Trong khu vực dữ liệu (bảng, form):**
- `rounded-xl` hoặc lớn hơn
- `shadow-lg` hoặc đổ bóng nặng hơn
- `bg-gradient`, `backdrop-blur`
- `border-dashed`
- **Nền trong suốt**

### **Trong toàn bộ ứng dụng:**
- Màu sắc sặc sỡ, không theo bảng màu
- Hiệu ứng chuyển động phức tạp
- Icon không đồng bộ

---

## **8. TRẠNG THÁI TƯƠNG TÁC** - *Rõ ràng và cứng nhắc*

### **Chọn ô/hàng (Selection)**
```css
bg-gray-200 border-2 border-blue-700
/* Giống Excel - focus rõ ràng */
```

### **Đang chỉnh sửa (Editing)**
```css
bg-white ring-2 ring-blue-700
/* "Cắt ra" khỏi bảng */
```

### **Vô hiệu hoá (Disabled)**
```css
bg-gray-100 text-gray-400 border-gray-200
/* Rõ ràng là không thể tương tác */
```

### **Thanh điều hướng**
- **Active**: `bg-gray-100 border-l-4 border-blue-700 text-gray-900`
- **Hover**: `bg-gray-50` (không transition)
- **Icon**: Màu đồng bộ với text

---

## ❗ **9. THANH CUỘN** – **SỬA HOÀN TOÀN (BREAKING CHANGE)**

### ❌ **XOÁ BỎ (V1.1 – iOS/Consumer Pattern)**

```css
/* Ẩn khi không cuộn, hiện mảnh khi cuộn */
scrollbar-thin scrollbar-thumb-gray-400 scrollbar-track-transparent
```

> **Lý do loại bỏ:**
> 
> - Auto-hide phá affordance
> - Scrollbar mảnh (~8px) vi phạm Fitts's Law
> - Không phù hợp với 100k+ rows
> - Phá định vị không gian dữ liệu

---

### ✅ **THAY THẾ – ENTERPRISE SCROLLBAR V1.2 (GROK STANDARD)**

```css
.enterprise-scroll-container {
  overflow-y: auto;
  overflow-x: auto;

  /* QUAN TRỌNG: không dùng contain: strict */
  contain: layout paint;

  /* Firefox */
  scrollbar-width: auto;
  scrollbar-color: #555555 #F0F0F0;
}

/* Webkit (Chrome, Edge, Tauri/WebView2) */
.enterprise-scroll-container::-webkit-scrollbar {
  width: 14px;
  height: 14px;
}

.enterprise-scroll-container::-webkit-scrollbar-track {
  background: #F0F0F0;
  border-left: 1px solid #E0E0E0;
}

.enterprise-scroll-container::-webkit-scrollbar-thumb {
  background-color: #555555;
  border-radius: 0;
}

.enterprise-scroll-container::-webkit-scrollbar-thumb:hover {
  background-color: #333333;
}
```

### 📌 **QUY TẮC ENTERPRISE SCROLLBAR**

- Native scrollbar **bắt buộc**
- Luôn hiển thị
- Rộng ~14–16px
- Không blur, không transparency
- Không JS hijacking
- Không fake scrollbar

> **Scrollbar = bản đồ không gian dữ liệu**, không phải trang sức.

---

## **10. KIỂM TRA ĐÁNH GIÁ** - *Litmus Tests*

Trước khi approve design, hỏi:

1. **UI có đang NHƯỜNG CHỖ cho dữ liệu không?**
2. **Sau 3 giờ làm việc, mắt có mỏi không?** (so với Excel)
3. **Có thể tìm dữ liệu mà không cần "cố gắng" không?**

### ➕ **BỔ SUNG V1.2 – KIỂM NGHIỆM ENTERPRISE (BẮT BUỘC)**

4. **Scrollbar có cho biết tôi đang ở đâu trong 100.000 dòng không?**
5. **Scroll có mượt vì native hay vì animation che giấu lag?**
6. **Nếu tắt CSS, bảng còn dùng được không?** (Enterprise Litmus Test)

---

## 🧱 **KẾT LUẬN KHÓA SPEC**

- V1.1 thất bại ở **Scrollbar & Data Density**
- V1.2 sửa bằng:
  - Virtualization-first
  - Native scrollbar
  - 32px row height
  - Tabular numerics
- Không còn mâu thuẫn nội tại giữa:
  - *Triết lý*
  - *Hiển thị*
  - *Hiệu năng*
  - *Nhận thức*

👉 **Spec này đã đủ cứng để làm chuẩn nội bộ hoặc design system thật.**
