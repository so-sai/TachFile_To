# BUSINESS RULES - VIETNAMESE CONSTRUCTION v1.0

**Version:** 1.0.0  
**Status:** Production-Ready  
**Last Updated:** 2025-12-25  
**Domain:** Vietnamese Quantity Surveying (QS)

---

## 🎯 Overview

This document defines the **business logic** specific to Vietnamese construction quantity surveying. It covers legacy font handling, terminology standardization, calculation rules, and validation constraints.

---

## 1. LEGACY FONT CONVERSION

### Problem Statement

Vietnamese construction documents from 1990-2015 often use legacy encodings:
- **TCVN3 (ABC)**: Government standard encoding
- **VNI**: Popular in Southern Vietnam
- **Unicode Composite**: Decomposed Vietnamese characters

These encodings render as gibberish in modern Unicode systems.

### Conversion Strategy

**Rust Implementation**: [`text/mod.rs`](file:///e:/DEV/TachFile_To/crates/tachfileto-core/src/text/mod.rs)

```rust
pub fn fix_vietnamese_text(input: &str) -> String {
    let mut result = input.to_string();
    
    // Step 1: Detect encoding
    let encoding = detect_encoding(&result);
    
    // Step 2: Convert to Unicode
    result = match encoding {
        Encoding::TCVN3 => tcvn3_to_unicode(&result),
        Encoding::VNI => vni_to_unicode(&result),
        Encoding::CompositeUnicode => normalize_unicode(&result),
        Encoding::Unicode => result // Already correct
    };
    
    // Step 3: Normalize Vietnamese diacritics
    normalize_vietnamese_diacritics(&result)
}
```

### Character Mapping Tables

#### TCVN3 → Unicode

| TCVN3 Byte | Unicode | Character | Example Word |
|------------|---------|-----------|--------------|
| `0xF5` | `U+00E1` | á | cát (sand) |
| `0xEF` | `U+00E0` | à | bàn (table) |
| `0xF9` | `U+1EA3` | ả | hỏa (fire) |
| `0xF2` | `U+00E3` | ã | lãnh đạo (leadership) |
| `0xF1` | `U+1EA1` | ạ | đạt (achieve) |

**Full mapping**: 256 characters defined in `TCVN3_MAP` constant.

#### VNI → Unicode

| VNI Sequence | Unicode | Character | Example Word |
|--------------|---------|-----------|--------------|
| `a1` | `U+00E1` | á | cát |
| `a2` | `U+00E0` | à | bàn |
| `a3` | `U+1EA3` | ả | hỏa |
| `a4` | `U+00E3` | ã | lãnh |
| `a5` | `U+1EA1` | ạ | đạt |

**Full mapping**: 134 sequences defined in `VNI_MAP` constant.

### Encoding Detection Heuristics

```rust
fn detect_encoding(text: &str) -> Encoding {
    let tcvn3_score = count_tcvn3_patterns(text);
    let vni_score = count_vni_patterns(text);
    let unicode_score = count_valid_unicode_vietnamese(text);
    
    // Return encoding with highest confidence score
    match (tcvn3_score, vni_score, unicode_score) {
        (t, _, _) if t > 0.7 => Encoding::TCVN3,
        (_, v, _) if v > 0.7 => Encoding::VNI,
        (_, _, u) if u > 0.9 => Encoding::Unicode,
        _ => Encoding::CompositeUnicode // Fallback
    }
}
```

---

## 2. CONSTRUCTION TERMINOLOGY

### Standard Component Categories

Vietnamese construction documents classify items into hierarchical categories:

```
Hạng mục (Category)
  └─ Công việc (Work Item)
      └─ Vật tư (Material)
```

#### Common Categories (Hạng mục)

| Vietnamese | English | Code |
|------------|---------|------|
| Phần thô | Structural work | PT |
| Phần hoàn thiện | Finishing work | HT |
| Hệ thống điện | Electrical system | D |
| Hệ thống nước | Plumbing system | N |
| Hệ thống PCCC | Fire protection | PCCC |

#### Common Work Items (Công việc)

| Vietnamese | English | Unit | Calculation Formula |
|------------|---------|------|---------------------|
| Đào đất | Excavation | m³ | Length × Width × Depth |
| Đổ bê tông | Concrete pouring | m³ | Volume from design |
| Xây gạch | Bricklaying | m² | Area - Openings |
| Sơn tường | Wall painting | m² | Area × 2 (two coats) |
| Lát gạch | Tile laying | m² | Area + 5% waste |

### Regional Terminology Variations

| North (Hà Nội) | South (TP.HCM) | Central (Đà Nẵng) | Standard |
|----------------|----------------|-------------------|----------|
| Xi măng | Xi măng | Xi măng | Cement |
| Cát vàng | Cát vàng | Cát vàng | Yellow sand |
| Đá 1×2 | Đá 1×2 | Đá dăm | Crushed stone |
| Sắt thép | Thép | Sắt | Steel rebar |

**Normalization Rule**: Always convert to **Standard** term in database.

---

## 3. QUANTITY CALCULATION RULES

### Volume Calculations

#### Concrete Volume

```rust
fn calculate_concrete_volume(
    length_m: f64,
    width_m: f64,
    height_m: f64,
    waste_factor: f64 // Typically 1.05 (5% waste)
) -> f64 {
    length_m * width_m * height_m * waste_factor
}
```

**Validation**:
- All dimensions must be > 0
- Waste factor must be in range [1.0, 1.15]

#### Excavation Volume

```rust
fn calculate_excavation_volume(
    top_area_m2: f64,
    bottom_area_m2: f64,
    depth_m: f64
) -> f64 {
    // Trapezoidal prism formula
    ((top_area_m2 + bottom_area_m2) / 2.0) * depth_m
}
```

### Area Calculations

#### Wall Area (with openings)

```rust
fn calculate_wall_area(
    length_m: f64,
    height_m: f64,
    openings: Vec<Opening>
) -> f64 {
    let gross_area = length_m * height_m;
    let opening_area: f64 = openings.iter()
        .map(|o| o.width * o.height)
        .sum();
    
    gross_area - opening_area
}
```

#### Tile Area (with waste)

```rust
fn calculate_tile_area(
    room_length_m: f64,
    room_width_m: f64,
    waste_percent: f64 // Typically 5-10%
) -> f64 {
    let net_area = room_length_m * room_width_m;
    net_area * (1.0 + waste_percent / 100.0)
}
```

---

## 4. CURRENCY & ROUNDING RULES

### Vietnamese Dong (VND) Formatting

```rust
fn format_vnd(amount: f64) -> String {
    // Round to nearest 1,000 VND
    let rounded = (amount / 1000.0).round() * 1000.0;
    
    // Format with thousand separators
    format!("{:,.0} đ", rounded)
}
```

**Examples**:
- `12,345,678.9` → `12,346,000 đ`
- `999,499` → `999,000 đ`
- `999,500` → `1,000,000 đ`

### Unit Price Precision

```rust
const UNIT_PRICE_PRECISION: u32 = 0; // No decimal places for VND
const QUANTITY_PRECISION: u32 = 2;   // 2 decimal places for quantities
```

---

## 5. TOLERANCE & VALIDATION

### Quantity Tolerances

| Item Type | Tolerance | Rationale |
|-----------|-----------|-----------|
| Concrete volume | ±2% | Formwork expansion, measurement error |
| Steel rebar | ±1% | Cutting waste, overlap |
| Brickwork | ±3% | Mortar joints, breakage |
| Tile area | ±5% | Cutting waste, pattern matching |
| Paint area | ±10% | Surface roughness, absorption |

### Validation Rules

```rust
fn validate_quantity(
    calculated: f64,
    measured: f64,
    tolerance_percent: f64
) -> ValidationResult {
    let diff_percent = ((measured - calculated) / calculated).abs() * 100.0;
    
    if diff_percent <= tolerance_percent {
        ValidationResult::Pass
    } else {
        ValidationResult::Fail {
            expected: calculated,
            actual: measured,
            diff_percent,
            max_allowed: tolerance_percent
        }
    }
}
```

---

## 6. UNIT CONVERSIONS

### Length Units

```rust
const MM_TO_M: f64 = 0.001;
const CM_TO_M: f64 = 0.01;
const M_TO_KM: f64 = 0.001;
```

### Area Units

```rust
const CM2_TO_M2: f64 = 0.0001;
const M2_TO_HA: f64 = 0.0001; // Hectare
```

### Volume Units

```rust
const L_TO_M3: f64 = 0.001;
const CM3_TO_M3: f64 = 0.000001;
```

### Weight Units

```rust
const G_TO_KG: f64 = 0.001;
const KG_TO_TON: f64 = 0.001;
```

---

## 7. STANDARD MATERIAL DENSITIES

Used for weight ↔ volume conversions:

| Material | Density (kg/m³) | Notes |
|----------|----------------|-------|
| Concrete (normal) | 2,400 | Structural concrete |
| Concrete (lightweight) | 1,800 | Aerated concrete |
| Steel rebar | 7,850 | Carbon steel |
| Brick (solid) | 1,800 | Clay brick |
| Brick (hollow) | 1,400 | Perforated brick |
| Sand (dry) | 1,600 | River sand |
| Gravel | 1,700 | Crushed stone |

```rust
fn calculate_material_weight(
    volume_m3: f64,
    material: Material
) -> f64 {
    volume_m3 * material.density_kg_per_m3()
}
```

---

## 8. EXTRACTION RULES FROM TABLES

### Table Structure Recognition

Vietnamese QS tables typically follow this structure:

```
┌────┬─────────────────┬──────┬────────┬─────────┬───────────┐
│ STT│   Tên hạng mục  │ ĐVT  │ Khối   │ Đơn giá │ Thành tiền│
│    │                 │      │ lượng  │         │           │
├────┼─────────────────┼──────┼────────┼─────────┼───────────┤
│ 1  │ Đào đất thủ công│  m³  │  150.5 │ 45,000  │6,772,500  │
└────┴─────────────────┴──────┴────────┴─────────┴───────────┘
```

### Column Mapping

```rust
struct QSTableSchema {
    stt_col: usize,           // Sequential number (STT)
    name_col: usize,          // Item name (Tên hạng mục)
    unit_col: usize,          // Unit (ĐVT)
    quantity_col: usize,      // Quantity (Khối lượng)
    unit_price_col: usize,    // Unit price (Đơn giá)
    total_col: usize          // Total (Thành tiền)
}
```

### Extraction Heuristics

```rust
fn detect_qs_table_columns(headers: Vec<String>) -> QSTableSchema {
    let stt_col = find_column_by_keywords(&headers, &["STT", "TT", "Số TT"]);
    let name_col = find_column_by_keywords(&headers, &["Tên", "Hạng mục", "Công việc"]);
    let unit_col = find_column_by_keywords(&headers, &["ĐVT", "Đơn vị"]);
    let quantity_col = find_column_by_keywords(&headers, &["Khối lượng", "SL", "Số lượng"]);
    let unit_price_col = find_column_by_keywords(&headers, &["Đơn giá", "Giá"]);
    let total_col = find_column_by_keywords(&headers, &["Thành tiền", "Tổng"]);
    
    QSTableSchema {
        stt_col,
        name_col,
        unit_col,
        quantity_col,
        unit_price_col,
        total_col
    }
}
```

---

## 9. VALIDATION FORMULAS

### Total Price Verification

```rust
fn verify_total_price(
    quantity: f64,
    unit_price: f64,
    stated_total: f64,
    tolerance_vnd: f64 // Typically 1,000 VND
) -> bool {
    let calculated_total = quantity * unit_price;
    (calculated_total - stated_total).abs() <= tolerance_vnd
}
```

### Sum Verification

```rust
fn verify_subtotal(
    items: Vec<QSItem>,
    stated_subtotal: f64,
    tolerance_vnd: f64
) -> bool {
    let calculated_subtotal: f64 = items.iter()
        .map(|item| item.total_price)
        .sum();
    
    (calculated_subtotal - stated_subtotal).abs() <= tolerance_vnd
}
```

---

## 10. COMMON ABBREVIATIONS

| Abbreviation | Vietnamese | English |
|--------------|-----------|---------|
| STT | Số thứ tự | Sequential number |
| ĐVT | Đơn vị tính | Unit of measurement |
| KL | Khối lượng | Quantity |
| ĐG | Đơn giá | Unit price |
| TT | Thành tiền | Total price |
| BT | Bê tông | Concrete |
| CT | Cốt thép | Steel rebar |
| XD | Xây dựng | Construction |
| PCCC | Phòng cháy chữa cháy | Fire protection |
| HTKT | Hệ thống kỹ thuật | Technical system |

---

## 11. TESTING DATA

### Sample Valid Entries

```json
[
  {
    "stt": "1",
    "name": "Đào đất thủ công",
    "unit": "m³",
    "quantity": 150.5,
    "unit_price": 45000,
    "total": 6772500
  },
  {
    "stt": "2",
    "name": "Đổ bê tông móng M200",
    "unit": "m³",
    "quantity": 25.3,
    "unit_price": 1850000,
    "total": 46805000
  }
]
```

### Sample Invalid Entries (for testing)

```json
[
  {
    "stt": "X",
    "name": "Invalid entry",
    "unit": "unknown",
    "quantity": -10,  // Negative quantity
    "unit_price": 0,  // Zero price
    "total": 999     // Doesn't match calculation
  }
]
```

---

## 12. FUTURE ENHANCEMENTS

### v1.1: Regional Dialect Support

Auto-detect and normalize regional terminology variations.

### v1.2: Historical Price Index

Adjust prices based on construction cost index over time.

### v1.3: Material Substitution Rules

Suggest equivalent materials when original is unavailable.

---

## APPENDIX: Vietnamese Character Reference

### Vowels with Diacritics

| Base | Acute | Grave | Hook | Tilde | Dot Below |
|------|-------|-------|------|-------|-----------|
| a | á | à | ả | ã | ạ |
| ă | ắ | ằ | ẳ | ẵ | ặ |
| â | ấ | ầ | ẩ | ẫ | ậ |
| e | é | è | ẻ | ẽ | ẹ |
| ê | ế | ề | ể | ễ | ệ |
| i | í | ì | ỉ | ĩ | ị |
| o | ó | ò | ỏ | õ | ọ |
| ô | ố | ồ | ổ | ỗ | ộ |
| ơ | ớ | ờ | ở | ỡ | ợ |
| u | ú | ù | ủ | ũ | ụ |
| ư | ứ | ừ | ử | ữ | ự |
| y | ý | ỳ | ỷ | ỹ | ỵ |

### Special Consonants

- **đ** (d with stroke): Unique to Vietnamese
- **Đ** (D with stroke): Capital form

---

**For implementation examples, see:**
- [`text/mod.rs`](file:///e:/DEV/TachFile_To/crates/tachfileto-core/src/text/mod.rs) - Font conversion
- [`engine/parser.py`](file:///e:/DEV/TachFile_To/backend/app/engine/parser.py) - Table extraction
