Aligned with: [MDS_ALIGNMENT.md]

# CORE 2: DOMAIN LAW CODIFICATION
## THE "STATE LAW" CONSTITUTION

**Status:** ACTIVE & ENFORCED
**Scope:** Regulatory Compliance (Nghị định 254/2025), Arithmetic Integrity, and Forensic Traceability.

> **Prime Directive:** "A technical error is a bug. A domain error is a violation of law. We do not catch bugs; we pronounce verdicts."

---

## 1. THE JURIDICAL MAPPING (TỪ "LỖI" SANG "LUẬT")

This section maps technical signals (from Calculator, Janitor, TableTruth) to Legal Rule IDs.

| Rule ID | Legal Name (Tên Pháp lý) | Technical Signal (Tín hiệu Kỹ thuật) | Severity | Verdict Message (Kết luận) |
| :--- | :--- | :--- | :--- | :--- |
| **R01** | **ARITHMETIC_INTEGRITY**<br>(Tính Chính xác Số học) | `abs(total - sum(items)) > epsilon` | **CRITICAL** | "Vi phạm R01: Sai lệch số học giữa tổng và chi tiết. Chênh lệch: {diff}." |
| **R02** | **PROVENANCE_TRACEABILITY**<br>(Tính Toàn vẹn Nguồn gốc) | `lineage.is_empty()` OR `bbox == null` | **CRITICAL** | "Vi phạm R02: Dữ liệu không có nguồn gốc chứng minh (Lineage/BBox bị thiếu)." |
| **R03** | **ENCODING_PURITY**<br>(Tính Xác thực Định dạng) | `encoding_status == Suspicious/Invalid` | **WARNING** | "Cảnh báo R03: Phát hiện ký tự lạ (Mojibake). Cần con người xác nhận." |
| **R04** | **DOMAIN_PLAUSIBILITY**<br>(Ngưỡng Hợp lý AEC) | `price < 0` OR `quantity < 0` | **CRITICAL** | "Vi phạm R04: Dữ liệu phi logic xây dựng (Giá trị âm)." |
| **R05** | **REGULATORY_COMPLIANCE**<br>(Tuân thủ Nghị định 254) | `missing_field IN [VAT, NgayNghiemThu]` | **INFO/WARN** | "Lưu ý R05: Hồ sơ thiếu trường thông tin bắt buộc theo NĐ 254: {field}." |

---

## 2. THE VERDICT SYSTEM (HỆ THỐNG PHÁN QUYẾT)

The `validation_engine` does not output "True/False". It outputs a **Written Verdict**.

### 2.1 Verdict Structure
```rust
pub struct LegalVerdict {
    pub rule_id: RuleId,        // R01..R05
    pub severity: Severity,     // INFO | WARNING | CRITICAL
    pub legal_text: String,     // "Vi phạm điều khoản..."
    pub technical_ref: String,  // "Row 45, Col 'Thành Tiền'"
    pub evidence: Option<String> // Link to crop or lineage ID
}
```

### 2.2 Aggregation Logic
- **1 CRITICAL** = **PROJECT REJECTED** (Hồ sơ bị trả về).
- **>0 WARNING** = **PROJECT TAINTED** (Hồ sơ cần giải trình/Human Audit).
- **0 Issues** = **PROJECT CLEAN** (Hồ sơ sạch).

---

## 3. IMPLEMENTATION STRATEGY

### 3.1 The "Judge" (`validation_engine.rs`)
- **Input:** `ProjectTruth` (from Calculator) + `TableTruth` (from Parser).
- **Process:** Iterates through the R01-R05 checklist.
- **Output:** `Vec<LegalVerdict>`.

### 3.2 The "Reporter"
- UI displays R01-R05 badges.
- Excel Export includes a "LEGAL_NOTES" column mapping rows to Rule IDs.

---

**END OF LAW**
