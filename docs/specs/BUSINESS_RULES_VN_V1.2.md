# BUSINESS RULES - VIETNAMESE CONSTRUCTION v1.2

**Version:** 1.2.0  
**Status:** Production-Ready  
**Last Updated:** 2025-12-25  
**Domain:** Vietnamese Quantity Surveying (QS)  
**References:** Nghị định 99/2021/NĐ-CP, Thông tư 12/2021/TT-BXD, FIDIC

---

## 🎯 Overview

Business logic for Vietnamese construction QS. Covers legacy fonts, terminology, VAT calculation, tolerance validation, and table extraction with fuzzy matching.

---

## 1. LEGACY FONT CONVERSION

### Encoding Detection Algorithm (v1.2 Enhanced)

```rust
fn count_tcvn3_patterns(text: &str) -> f64 {
    // Pass 1: Frequency analysis (bytes 0xE0-0xFF)
    let matches = text.bytes().filter(|&b| b >= 0xE0 && b <= 0xFF).count();
    matches as f64 / text.len() as f64
}

fn count_vni_patterns(text: &str) -> f64 {
    // Regex: alphabet + digit (a1, e2, o3...)
    let regex = regex::Regex::new(r"[aeiouyAEIOUY][1-8]").unwrap();
    regex.find_iter(text).count() as f64 / text.len() as f64
}

fn detect_encoding(text: &str) -> Encoding {
    let tcvn3_score = count_tcvn3_patterns(text);
    let vni_score = count_vni_patterns(text);
    
    // Threshold: >10% specific patterns
    match (tcvn3_score, vni_score) {
        (t, _) if t > 0.10 => Encoding::TCVN3,
        (_, v) if v > 0.10 => Encoding::VNI,
        _ => Encoding::Unicode
    }
}
```

### Character Mapping (Unchanged from v1.0)
- TCVN3: 256 bytes mapped in `TCVN3_MAP`
- VNI: 134 sequences mapped in `VNI_MAP`

---

## 2. TERMINOLOGY & NORMALIZATION

### Term Preservation Policy (NEW in v1.2)

```rust
enum ExportTermStrategy {
    PreserveOriginal,  // Keep raw: "Sắt thép"
    UseStandard,       // Convert: "Steel rebar"
    ShowBoth,          // Hybrid: "Sắt thép (Steel rebar)"
}

struct TermNormalization {
    original: String,      
    normalized: String,    
    region: Region,        
    confidence: f64  // If <0.8, flag for manual review
}
```

**Default**: `PreserveOriginal` for Excel export. Configurable via YAML.

---

## 3. NUMBER FORMATTING & ROUNDING

### VND Currency Rules

```rust
const UNIT_PRICE_PRECISION: u32 = 0;   // No decimals
const QUANTITY_PRECISION: u32 = 2;     // 2 decimals
const FINAL_ROUND_TO: i32 = -3;        // Nearest 1,000 VND
```

### VAT Calculation (NEW in v1.2)

```rust
fn calculate_with_vat(subtotal: Decimal, vat_rate: Decimal) -> PriceCalculation {
    // Step 1: VAT (round to 0 decimals)
    let vat_amount = (subtotal * vat_rate)
        .round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero);
    
    // Step 2: Grand total (round to nearest 1,000)
    let grand_total = (subtotal + vat_amount)
        .round_dp_with_strategy(-3, RoundingStrategy::MidpointAwayFromZero);
    
    PriceCalculation { subtotal, vat_rate, vat_amount, grand_total }
}
```

**Example**: `12,345,678 VND → VAT 1,234,568 → Total 13,580,000 VND`

---

## 4. TOLERANCE & VALIDATION

### Edge Case Handling (NEW in v1.2)

```rust
fn validate_quantity_advanced(
    calculated: f64, 
    measured: f64, 
    tolerance_percent: f64,
    min_absolute_threshold: f64  // Default: 0.1
) -> ValidationResult {
    // Near-zero: use absolute diff instead of %
    if calculated.abs() < min_absolute_threshold {
        if (measured - calculated).abs() <= min_absolute_threshold {
            return ValidationResult::Pass;
        } else {
            return ValidationResult::Fail { reason: "Absolute diff exceeded" };
        }
    }
    
    // Standard % tolerance
    let diff_percent = ((measured - calculated) / calculated).abs() * 100.0;
    if diff_percent <= tolerance_percent {
        ValidationResult::Pass
    } else {
        ValidationResult::Fail { reason: "Percent diff exceeded" }
    }
}
```

### Standard Tolerances

| Item Type | Tolerance | Min Absolute |
|-----------|-----------|--------------|
| Concrete  | ±2%       | 0.01 m³      |
| Steel     | ±1%       | 0.1 kg       |
| Brick     | ±3%       | 0.1 m²       |
| Tile      | ±5%       | 0.01 m²      |

---

## 5. TABLE EXTRACTION

### Fuzzy Column Matching (NEW in v1.2)

```rust
fn find_column_by_keywords_fuzzy(
    headers: &[String], 
    keywords: &[&str], 
    threshold: f64  // Default: 0.8
) -> Option<usize> {
    headers.iter().enumerate().filter_map(|(i, h)| {
        // Jaro-Winkler similarity
        let similarity = keywords.iter()
            .map(|k| jaro_winkler(h, k))
            .max()
            .unwrap_or(0.0);
        if similarity >= threshold { Some((i, similarity)) } else { None }
    }).max_by(|a, b| a.1.partial_cmp(&b.1)).map(|(i, _)| i)
}
```

**Example**: Header "Khói lượng" → matches "Khối lượng" (similarity ~0.84)

### QS Table Schema Detection

```rust
fn detect_qs_table_columns(headers: Vec<String>) -> QSTableSchema {
    let stt = find_column_by_keywords_fuzzy(&headers, &["STT", "TT"], 0.8);
    let name = find_column_by_keywords_fuzzy(&headers, &["Tên", "Hạng mục"], 0.8);
    let unit = find_column_by_keywords_fuzzy(&headers, &["ĐVT", "Đơn vị"], 0.8);
    let quantity = find_column_by_keywords_fuzzy(&headers, &["Khối lượng", "SL"], 0.8);
    let price = find_column_by_keywords_fuzzy(&headers, &["Đơn giá", "Giá"], 0.8);
    let total = find_column_by_keywords_fuzzy(&headers, &["Thành tiền", "Tổng"], 0.8);
    
    QSTableSchema { stt, name, unit, quantity, price, total }
}
```

---

## 6. UNIT CONVERSIONS & DENSITIES

### Standard Conversions
```rust
const MM_TO_M: f64 = 0.001;
const CM_TO_M: f64 = 0.01;
const L_TO_M3: f64 = 0.001;
const KG_TO_TON: f64 = 0.001;
```

### Material Densities (kg/m³)

| Material | Density | Notes |
|----------|---------|-------|
| Concrete | 2,400   | Normal structural |
| Steel    | 7,850   | Carbon steel |
| Brick    | 1,800   | Solid clay |
| Sand     | 1,600   | Dry river |

---

## 7. ABBREVIATIONS

| Code | Vietnamese | English |
|------|-----------|---------|
| STT  | Số thứ tự | Seq. number |
| ĐVT  | Đơn vị tính | Unit |
| KL   | Khối lượng | Quantity |
| ĐG   | Đơn giá | Unit price |
| TT   | Thành tiền | Total |
| BT   | Bê tông | Concrete |
| CT   | Cốt thép | Steel |

---

## 8. TEST CASES (v1.2 Enhanced)

### Edge Case Tests

```json
[
  {"name": "Near-zero", "calculated": 0.0, "measured": 0.05, "threshold": 0.1, "expected": "Pass"},
  {"name": "Small quantity", "calculated": 0.01, "measured": 0.02, "threshold": 0.01, "expected": "Pass"},
  {"name": "VAT rounding", "subtotal": 12345678, "vat": 1234568, "total": 13580000}
]
```

### Fuzzy Header Test

```json
{"input": ["Khói lượng"], "target": "Khối lượng", "similarity": 0.84, "expected": "Match"}
```

---

**Implementation**: See [`text/mod.rs`](file:///e:/DEV/TachFile_To/crates/tachfileto-core/src/text/mod.rs)
