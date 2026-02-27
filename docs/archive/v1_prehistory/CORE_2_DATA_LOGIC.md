Aligned with: [MDS_ALIGNMENT.md]

# CORE 2: DATA CONSISTENCY LOGIC
## THE "TRUTH VERIFICATION" SPECIFICATION

**Status:** ACTIVE & ENFORCED
**Scope:** Arithmetic Integrity, Data Provenance, and Cross-Source Consistency.

> **Prime Directive:** "We do not judge the business. We verify the data. If the numbers don't match, or the source is missing, we flag it as a Data Fact."

---

## 1. THE VERIFICATION MATRIX (MA TRẬN KIỂM TRA)

This section maps technical anomalies to Data Verdicts.

| Rule ID | Technical Name (Tên Kỹ thuật) | Check Condition (Điều kiện Kiểm tra) | Verdict Message (Kết luận) |
| :--- | :--- | :--- | :--- |
| **R01** | **ARITHMETIC_INCONSISTENCY**<br>(Bất nhất Số học) | `abs(Total - Sum(Details)) > epsilon` | "Inconsistent: Calculated sum {calc} != Reported value {val}." |
| **R02** | **LINEAGE_BREAK**<br>(Gãy chuỗi Nguồn gốc) | `lineage.is_empty()` OR `bbox == null` | "Untraceable: Data row lacks forensic evidence (PDF Crop/Excel Cell)." |
| **R03** | **CHARACTER_CORRUPTION**<br>(Lỗi Định dạng Ký tự) | `encoding_status == Suspicious/Invalid` | "Corrupted: Potential Mojibake or encoding error detected." |
| **R04** | **VALUE_ANOMALY**<br>(Bất thường Giá trị) | `price < 0` OR `qty < 0` OR `(price > 0 AND qty == 0 AND total > 0)` | "Anomaly: Negative value or logical impossibility." |
| **R05** | **CROSS_SOURCE_MISMATCH**<br>(Lệch Nguồn Đối chiếu) | `PDF.value != Excel.value` | "Mismatch: Source PDF says {pdf_val} but Excel says {xls_val}." |

---

## 2. THE VERDICT SYSTEM (HỆ THỐNG KẾT LUẬN)

The `validation_engine` serves as a high-speed filter logic.

### 2.1 Verdict Structure
```rust
pub enum ViolationType {
    MathError,      // R01
    Traceability,   // R02
    Encoding,       // R03
    Anomaly,        // R04
    Mismatch,       // R05
}

pub struct DataVerdict {
    pub violation: ViolationType,
    pub severity: u8,          // 1: Info, 2: Warning, 3: Critical
    pub message: String,       // Technical description of the delta
    pub cell_ref: String,     // Global ID or Coordinate
}
```

### 2.2 Aggregation Logic
- **Any Critical Violation** -> The specific Table/Row is marked **INVALID**.
- **The Project** is simply a container of these Facts. It is "Clean" only if 0 Violations found.

---

## 3. IMPLEMENTATION STRATEGY

### 3.1 The "Verifier" (`validation_engine.rs`)
- **Input:** `TableTruth` (and potentially comparison pairs).
- **Process:**
    1. Re-calculate rows (R01).
    2. Check metadata presence (R02).
    3. Check encoding flags (R03).
    4. Scan for negatives (R04).
- **Output:** `Vec<DataVerdict>`.

### 3.2 No "Legal" Language
- Avoid: "Vi phạm pháp luật", "Không tuân thủ nghị định".
- Use: "Sai lệch", "Không khớp", "Thiếu dữ liệu".

---

**END OF LOGIC**
