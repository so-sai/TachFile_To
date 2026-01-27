# TachFileTo – MDS Alignment Document

**MDS Reference:** MDS-ELITE10-2026-v1.0  
**Status:** OPTIONAL MODULE  
**Last Updated:** 2026-01-12

---

## I. ECOSYSTEM CLASSIFICATION

> **Per MDS §II.3:** TachFileTo là **HEAVY INGESTION TOOL (OPTIONAL)**

```
┌──────────────────┐
│    iron_core     │  ← BRAIN (Rules/Policy)
│ (Rules / Policy) │
└───────▲──────────┘
        │
┌──────────────┐
│  AutoQSVN    │  ← DEFAULT (Word/Excel)
│ (Word/Excel) │
└──────▲───────┘
       │ (OPTIONAL)
┌──────────────┐
│ TachFileTo   │  ← YOU ARE HERE
│ PDF + AI     │
│  OPTIONAL    │
└──────────────┘
```

---

## II. EXISTENCE CONDITIONS

TachFileTo **chỉ tồn tại** khi **đồng thời có:**

| Điều kiện | Mô tả |
|-----------|-------|
| ✅ PDF nặng / scan | File >50MB, scan chất lượng thấp |
| ✅ Cần OCR / AI | Không thể extract bằng phương pháp thông thường |
| ✅ Chi phí xử lý cao | Yêu cầu GPU/compute resources đáng kể |

**Nếu thiếu bất kỳ điều kiện nào → Dùng AutoQSVN thay thế.**

---

## III. LAW COMPLIANCE MATRIX

| Law | Yêu cầu | TachFileTo Compliance |
|-----|---------|----------------------|
| **LAW-01** | Single Responsibility | ✅ Chỉ làm ingestion/extraction |
| **LAW-02** | No Reverse Dependency | ✅ Không gọi ngược iron_core |
| **LAW-03** | Payment Safety | ✅ Không tham gia payment flow |
| **LAW-04** | Optional Module Rule | ✅ Không xuất hiện trong critical path |
| **LAW-05** | Ecosystem First | ✅ Tuân thủ BOUNDARY_MANIFEST |

---

## IV. ALLOWED OPERATIONS (Per MDS)

✅ **Được phép:**
- Là plugin / sidecar / sandbox
- Làm sạch & trích xuất dữ liệu PDF
- OCR processing
- AI-assisted extraction (non-critical)

---

## V. FORBIDDEN OPERATIONS (Per MDS)

❌ **Bị cấm:**
- Tham gia flow thanh toán bắt buộc
- Thay thế AutoQSVN xử lý Word/Excel
- Ghi logic nghiệp vụ
- Trở thành dependency bắt buộc

---

## VI. INTEGRATION PATTERN

```
[User] → [TachFileTo] → [Cleaned JSON] → [AutoQSVN hoặc iron_core]
                ↑
         OPTIONAL PATH
         (Bypass if not needed)
```

**Key Principle:** Hệ thống phải hoạt động bình thường **ngay cả khi TachFileTo không tồn tại**.

---

## VII. REFERENCES

- [GOVERNANCE/MDS.md](file:///e:/DEV/elite10-ecosystem/GOVERNANCE/MDS.md) – Master Design Spec
- [GOVERNANCE/ARCH_LAWS.md](file:///e:/DEV/elite10-ecosystem/GOVERNANCE/ARCH_LAWS.md) – Architecture Laws
- [docs/BOUNDARY_MANIFEST.md](file:///e:/DEV/elite10-ecosystem/app-tool-TachFileTo/docs/BOUNDARY_MANIFEST.md) – Local boundaries

---

**Document Authority:** Elite 10 Ecosystem Architecture Board
