# 📉 Extraction Performance & Integrity Report

**Module Name:** [e.g., Docling-Embedded-V1]  
**Commit Hash:** [git-hash]  
**Test Date:** [YYYY-MM-DD]  
**Reviewer:** Architect / Founder

---

## 1. Môi trường thử nghiệm (Test Environment)

| Parameter | Value |
|:----------|:------|
| **CPU** | Intel i5-11400H (limit to 4 cores for simulation) |
| **RAM** | 16GB (limit to 2GB for test) |
| **OS** | Windows 11 (Tauri Target) / Linux Musl |
| **Python Runtime** | Embedded via PyOxidizer (Python 3.14.x) |

---

## 2. Dữ liệu mẫu (Test Corpus)

| ID | Loại file | Số trang | Dung lượng | Đặc điểm |
|:---|:----------|:---------|:-----------|:---------|
| TC-01 | PDF | 50 | 5MB | Văn bản thuần, 2 cột |
| TC-02 | PDF | 100 | 15MB | Có bảng biểu phức tạp, Scan chất lượng thấp |
| TC-03 | DOCX | 30 | 2MB | Hợp đồng xây dựng nhiều nested tables |

---

## 3. Chỉ số Hiệu năng (SLA Gate)

> [!WARNING]
> Mọi chỉ số **FAIL** đồng nghĩa với việc **REJECT ngay lập tức**.

| Metric | Target | Result (Avg) | Status |
|:-------|:-------|:-------------|:-------|
| **Cold Start** (Init Runtime) | ≤ 5.0s | `___` s | `[PASS/FAIL]` |
| **Warm Parse** (TC-02) | ≤ 1.0s | `___` s | `[PASS/FAIL]` |
| **Peak RAM** | ≤ 800MB | `___` MB | `[PASS/FAIL]` |
| **CPU Usage** | ≤ 60% (4 cores) | `___` % | `[PASS/FAIL]` |

### Rejection Thresholds

| Scenario | Action |
|:---------|:-------|
| Warm parse > 3s | **IMMEDIATE REJECT** |
| Cold start > 10s | **IMMEDIATE REJECT** |
| Peak RAM > 1.5GB | **IMMEDIATE REJECT** |

---

## 4. Kiểm tra Tính Quyết định (Determinism)

**Method:** Chạy cùng 1 file TC-02 **10 lần liên tiếp**.

### Hash Check (SHA-256)

| Run | Hash |
|:----|:-----|
| 1 | `[hash_01]` |
| 2 | `[hash_02]` |
| 3 | `[hash_03]` |
| 4 | `[hash_04]` |
| 5 | `[hash_05]` |
| 6 | `[hash_06]` |
| 7 | `[hash_07]` |
| 8 | `[hash_08]` |
| 9 | `[hash_09]` |
| 10 | `[hash_10]` |

**Identical hashes:** `___/10`

**Conclusion:** `[PASS/FAIL]`

---

## 5. Kiểm tra Integrity (Ground Truth)

### 5.1 Schema Validation

| Check | Status |
|:------|:-------|
| JSON schema valid | `[PASS/FAIL]` |
| All required fields present | `[PASS/FAIL]` |
| No null values in critical fields | `[PASS/FAIL]` |

### 5.2 Financial Data Accuracy

**Test file:** Hợp đồng xây dựng với giá trị đã biết trước.

| Field | Expected | Extracted | Match |
|:------|:---------|:----------|:------|
| Tổng giá trị HĐ | `___` VNĐ | `___` VNĐ | `[YES/NO]` |
| VAT (10%) | `___` VNĐ | `___` VNĐ | `[YES/NO]` |
| Số lượng item | `___` | `___` | `[YES/NO]` |

**Deviation tolerance:** 0% (must be exact match)

---

## 6. Binary Distribution Check

| Check | Status |
|:------|:-------|
| Single binary (no external Python) | `[PASS/FAIL]` |
| Binary size | `___` MB |
| Runs without network | `[PASS/FAIL]` |
| SHA-256 of binary | `[hash]` |

---

## 7. ARCHITECT VERDICT

| Criteria | Status |
|:---------|:-------|
| Performance SLA | `[PASS/FAIL]` |
| Determinism | `[PASS/FAIL]` |
| Integrity | `[PASS/FAIL]` |
| Distribution | `[PASS/FAIL]` |

### Final Decision

```
[ ] APPROVED - Ready for Iron Core merge
[ ] REJECTED - See notes below
[ ] CONDITIONAL - Requires fixes listed below
```

### Notes

```
[Reviewer notes here]
```

---

**Reviewer Signature:** ________________________  
**Date:** ________________________
