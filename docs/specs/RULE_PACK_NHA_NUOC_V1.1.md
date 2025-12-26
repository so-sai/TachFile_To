# RULE_PACK_NHA_NUOC.md - V1.1
## SPEC DUY NHẤT CHO IRON CORE ENGINE

**Version:** 1.1.0  
**Effective:** 26/12/2025  
**Status:** ĐÃ KHÓA - READY FOR CODING

---

## 📌 1. MỤC ĐÍCH DUY NHẤT

Phát hiện **SAI PHÁP LÝ – SAI TIỀN – THIẾU BẰNG CHỨNG** trong hồ sơ thanh toán/quyết toán dự án Nhà nước theo cách **deterministic, không suy đoán**.

---

## 🧠 2. NGUYÊN TẮC THÉP (BẤT BIẾN)

1. **Deterministic tuyệt đối**: Cùng input → cùng output
2. **Zero False Fatal**: Hồ sơ hợp lệ không bao giờ bị Fatal
3. **Traceable**: Mọi violation có Rule ID + vị trí + lý do
4. **Three-Level Validation**: Row / Group / Document
5. **UI không diễn giải**: Chỉ hiển thị kết quả rule

---

## 🗂️ 3. SCHEMA BẮT BUỘC

### 3.1 HangMuc (Row-level)
```rust
enum TrangThaiHangMuc {
    ChinhThuc,      // Đã phê duyệt
    TamTinh,        // Tạm tính
    ChoPheDuyet,    // Chờ duyệt
}

struct HangMuc {
    ma_hang_muc: String,
    mo_ta: String,
    trang_thai: TrangThaiHangMuc,      // BẮT BUỘC - không default
    ma_dinh_muc: Option<String>,       // Required nếu ChinhThuc
    
    khoi_luong: f64,
    don_gia: f64,
    thanh_tien_thuc_te: f64,
    
    bien_ban_nghiem_thu: bool,
    evidence_links: Vec<String>,
    giai_trinh_lech: Option<String>,   // Bắt buộc nếu lệch >3%
    
    nhom: String,                      // Nhóm thanh toán
}
```

### 3.2 VAT Policy (NEW - V1.1)
```rust
enum VatCategory {
    ChinhSach0Percent,      // Xuất khẩu
    UuDai5Percent,          // Y tế, giáo dục
    TieuChuan10Percent,     // Xây dựng thông thường
    GiamTamThoi8Percent,    // Giảm tạm thời 2025-2026
}

struct VatPolicyConfig {
    vat_rate_chuan: 0.10,
    vat_rate_hien_hanh: 0.08,
    policy_period_start: "2025-07-01",
    policy_period_end: "2026-12-31",
}
```

### 3.3 DocumentMetadata (UPDATED)
```rust
struct DocumentMetadata {
    // Hợp đồng
    gia_tri_hop_dong_goc: f64,
    tong_phu_luc_hieu_luc: f64,    // Chỉ cộng phụ lục đã phê duyệt
    gia_tri_hien_hanh: f64,        // = goc + tong_phu_luc_hieu_luc
    
    // Thanh toán
    ngay_thanh_toan: DateTime,     // NEW - để xác định VAT rate
    tong_thanh_tien: f64,
    tong_vat: f64,
    
    // VAT Category (NEW)
    vat_category: VatCategory,
    vat_category_evidence: Option<String>,  // Link văn bản pháp lý
    
    // Tạm ứng
    tong_tam_ung: f64,
    tam_ung_da_thu_hoi: f64,
    
    // Lũy kế
    luy_ke_thanh_toan_ky_truoc: f64,
    luy_ke_thanh_toan: f64,        // = ky_truoc + tong_thanh_tien
}
```

---

## ⚖️ 4. DANH SÁCH RULE (ĐÃ KHÓA)

### 4.1 Row-level Rules
| ID | Severity | Điều kiện | Lý do |
|----|----------|-----------|-------|
| R01 | 🔴 Fatal | `trang_thai == ChinhThuc && ma_dinh_muc.is_none()` | "Hạng mục chính thức [{ma}] thiếu mã định mức" |
| R02 | 🟡 Warning | `trang_thai != ChinhThuc && ma_dinh_muc.is_none()` | "Hạng mục tạm tính [{ma}] chưa có mã định mức" |
| R03 | 🔴 Fatal | `abs(tt_thuc_te - (kl * dg)) > epsilon_kldgtt` | "Dòng [{ma}]: KL×ĐG ≠ TT (chênh: {diff})" |
| R04 | 🔴 Fatal | `kl > 0 && !bbnt` | "Dòng [{ma}] có KL >0 nhưng thiếu BBNT" |
| R05 | 🟡 Warning | `kl > 0 && evidence.is_empty()` | "Dòng [{ma}] thiếu evidence ảnh/video" |

### 4.2 Document-level Rules (UPDATED)
| ID | Severity | Điều kiện | Lý do |
|----|----------|-----------|-------|
| D01 | 🔴 Fatal | `luy_ke > gia_tri_hien_hanh` | "Lũy kế ({luy_ke}) vượt giá trị HĐ ({gia_tri})" |
| D02 | 🔴 Fatal | `abs(tong_vat - tong_thanh_tien * vat_rate(category, ngay)) > epsilon_vat` | "VAT khai báo ≠ {rate}% theo [{category}]" |
| D02b | 🟢 Info | `category == GiamTamThoi8Percent` | "Đang áp dụng giảm 2% VAT (2025-2026)" |
| D03 | 🟡 Warning | `tong_tam_ung - da_thu_hoi > 0` | "Còn {con_lai} tạm ứng chưa thu hồi" |
| D03b | 🔴 Fatal | `da_thu_hoi > tong_tam_ung` | "Thu hồi ({thu}) > Tổng ứng ({ung})" |
| D04 | 🟡 Warning | `phu_luc.evidence_van_ban.is_none()` | "Phụ lục {so_phu_luc} chưa có văn bản phê duyệt" |

**Quan trọng:** Bất kỳ Fatal nào → Dashboard **ĐỎ TOÀN BỘ**.

---

## ⚙️ 5. THAM SỐ MẶC ĐỊNH
```yaml
epsilon_kldgtt: 1.0          # Sai số KL×ĐG vs TT
epsilon_vat: 100.0           # Sai số VAT
lech_warning_threshold: 0.03 # 3%
vat_config:                  
  rate_chuan: 0.10
  rate_hien_hanh: 0.08
  period_start: "2025-07-01"
  period_end: "2026-12-31"
```

---

## 📊 6. OUTPUT FORMAT
```rust
struct ValidationResult {
    trang_thai: "DO" | "VANG" | "XANH",
    violations: Vec<Violation>,
    nguon_du_lieu: {          // NEW - traceability
        file_name: String,
        hash_md5: String,
        extracted_at: DateTime
    }
}

struct Violation {
    rule_id: String,          // "R01", "D02"
    severity: "FATAL" | "WARNING" | "INFO",
    vi_tri: String,           // "Document", "Row:A.1.1"
    ly_do: String,
    actual: Option<f64>,
    expected: Option<f64>,
}
```

---

## 🧪 7. TEST CASES BẮT BUỘC (6 CASES)

### Case 1: PASS sạch
- Đủ mã định mức, KL×ĐG đúng
- Có BBNT + evidence
- Lũy kế ≤ giá trị HĐ
- VAT đúng rate

→ **XANH**

### Case 2: FAIL Fatal (D01)
- Lũy kế vượt HĐ 1%

→ **ĐỎ**

### Case 3: WARNING hợp lệ
- Hạng mục tạm tính thiếu mã

→ **VÀNG**

### Case 4: VAT 8% hợp lệ (NEW)
- Ngày: 2025-12-26 (trong policy period)
- Category: `TieuChuan10Percent`
- VAT = 8%

→ **XANH** + Info D02b

### Case 5: VAT 0% hợp lệ (NEW)
- Category: `ChinhSach0Percent`
- Evidence: Thông báo xuất khẩu

→ **XANH**

### Case 6: Thu hồi tạm ứng quá mức (NEW)
- Tổng ứng: 500M
- Thu hồi: 550M

→ **ĐỎ** (D03b)

---

## 📄 8. XỬ LÝ PDF (POLICY)

### Giai đoạn hiện tại (Alpha):
- PDF chỉ là **evidence container**
- Đính kèm vào `evidence_links`
- **Không parse, không OCR**

### Nguyên tắc vàng:
> "Nếu số liệu chỉ đến từ PDF không có Excel đối chiếu → chỉ Warning, không Fatal"

---

## 📋 9. CHANGELOG V1.1

**Added:**
- VAT dynamic với 4 categories (0%, 5%, 8%, 10%)
- Rule D02b (Info VAT policy)
- Rule D03b (Fatal thu hồi quá mức)
- Rule D04 (Warning phụ lục thiếu evidence)
- Field `ngay_thanh_toan` cho VAT rate
- Test case 4,5,6

**Changed:**
- D02: Hard-code 0.10 → dynamic `vat_rate_by_category()`
- `gia_tri_hien_hanh`: Chỉ cộng phụ lục có hiệu lực

**Fixed:**
- Zero false fatal cho VAT 0%, 5%, 8%
- Zero false pass cho phụ lục chưa duyệt

---

## 🚀 10. BẮT ĐẦU CODE

**Spec này đã khóa.** Mọi thay đổi phải:
1. Tăng version (v1.2.0, v2.0.0)
2. Update trực tiếp file này
3. Có changelog rõ ràng

**Next action ngay bây giờ:**
1. Implement `validation_engine.rs` theo SPEC
2. Viết 6 integration tests cho 6 cases
3. Không thêm tài liệu mới

---

*Tài liệu duy nhất này đủ để code Iron Core Engine. Bắt đầu.* 🏗️

---

**SPEC KẾT THÚC TẠI ĐÂY**
