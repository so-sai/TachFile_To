Aligned with: [MDS_ALIGNMENT_V1.0.md]

# IRON TRUTH CONTRACT V1.0

**Mission:** 017 - Deep Table Logic  
**Status:** CANONICAL & IMMUTABLE SPECIFICATION  
**Version:** 1.0.0  
**Created:** 2026-01-31

---

## 0. WHY THIS DOCUMENT EXISTS (UNIFICATION NOTE)

This document merges and supersedes two previously separate contracts:
1. **ProjectTruth** (Iron Core ↔ Dashboard)
2. **TableTruth** (Deep Table Logic / Polars / Docling)

**Reason for merge:**
- There is one single source of truth: **Iron Core**
- Tables are **foundational truth** → Project truth is **derived truth**
- Splitting documents creates artificial boundaries and future drift

**Rule:** There is now **ONE constitution**. Any previous standalone contract is deprecated.

---

## I. CONSTITUTIONAL PRINCIPLES (GLOBAL)

These principles govern **all data flowing out of Iron Core**.

### 1. Zero Business Logic (Hard Boundary)
- ❌ No BOQ interpretation
- ❌ No QS judgment
- ❌ No cost correctness validation
- ❌ No AI / heuristic inference
- ✅ Only **structural, arithmetic, and rule-based transformations**

Iron Core declares **truth of data**, not truth of meaning.

### 2. Deterministic Absolute
- Same input → same output (**byte-identical**)
- No randomness
- No fuzzy thresholds
- All transformations **reversible or auditable**

If two runs differ → **BUG**, not variance.

### 3. Contract-First, Reject-by-Default
- Data without a valid contract → **REJECTED**
- No silent fixing
- No schema inference

**Rejection is safer than corruption.**

---

## II. LAYERED TRUTH MODEL

1. **Raw File** (PDF / Excel)
2. **Docling v2** (Signal Provider)
3. **TableTruth** (Structural Truth)
4. **Polars DataFrames** (Execution Engine)
5. **ProjectTruth** (Aggregated Declarative Truth)
6. **Dashboard UI** (Pure Renderer)

---

## III. TABLE TRUTH CONTRACT (FOUNDATION)

### 3.1 TableTruth Structure

```rust
pub struct TableTruth {
    pub table_id: String,          // SHA-256(source + page + bbox)
    pub source_file: PathBuf,
    pub source_page: u32,

    pub schema: TableSchema,
    pub rows: Vec<TableRow>,

    pub extraction_meta: ExtractionMeta,
    pub bbox: BoundingBox,
}
```

### 3.2 TableSchema

```rust
pub struct TableSchema {
    pub columns: Vec<ColumnDef>,
    pub row_count: usize,
    pub col_count: usize,
}

pub struct ColumnDef {
    pub name: String,
    pub dtype: DataType,           // Int | Float64 | Utf8 | Date
    pub unit: Option<String>,
    pub nullable: bool,
}
```

### 3.3 Cell-Level Truth

```rust
pub struct TableCell {
    pub row_idx: usize,
    pub col_idx: usize,
    pub value: CellValue,
    pub bbox: BoundingBox,
    pub confidence: f32,
    pub source_text: String,
}
```

### 3.4 Rejection Rules (Hard)

A table is **rejected** if ANY condition is met:
- ❌ Merged cells without deterministic mapping
- ❌ Multi-row headers without explicit rule
- ❌ Rotated or nested tables
- ❌ Confidence < 0.7 for any cell
- ❌ > 10,000 rows or > 100 columns

**Rejected tables do not propagate upward.**

---

## IV. NORMALIZATION (DETERMINISTIC ONLY)

### 4.1 Header Normalization
- lowercase
- remove accents
- trim
- spaces → `_`

### 4.2 Unit Normalization
- Exact string mapping only (`m2` → `m²`, `m3` → `m³`, etc.)

### 4.3 Numeric Normalization
- `(raw * 1000.0).round() / 1000.0`

---

## V. POLARS INTEGRATION (EXECUTION LAYER)

- **Polars executes, never decides**
- Schema is **explicit**
- No inference, no coercion
- `Series::new("khoi_luong", &values)`

**Forbidden:**
- `infer_schema`
- silent `fill_null`

---

## VI. PROJECT TRUTH CONTRACT (DERIVED, DECLARATIVE)

### 6.1 ProjectTruth Schema (UI-Facing)

```typescript
interface ProjectTruth {
  project_name: string;
  last_updated: string;
  data_source: string;

  project_status: "SAFE" | "WARNING" | "CRITICAL";
  status_reason: string;

  financials: Financials;
  deviation: DeviationSummary;
  top_risks: RiskItem[];
  pending_actions: ActionItem[];
  metrics: SystemMetrics;
}
```

**Important:** ProjectTruth is **pure output**. UI must not compute anything.

### 6.2 Rust Implementation (Canonical)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTruth {
    pub project_name: String,
    pub last_updated: String,  // ISO 8601
    pub data_source: String,

    pub project_status: ProjectStatus,
    pub status_reason: String,

    pub financials: Financials,
    pub deviation: DeviationSummary,
    pub top_risks: Vec<RiskItem>,
    pub pending_actions: Vec<ActionItem>,
    pub metrics: SystemMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectStatus {
    Safe,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Financials {
    pub total_cost: f64,
    pub total_paid: f64,
    pub remaining: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviationSummary {
    pub percentage: f64,
    pub absolute: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskItem {
    pub description: String,
    pub severity: u8,  // 1-10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub description: String,
    pub deadline: Option<String>,  // ISO 8601
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub table_count: usize,
    pub row_count: usize,
    pub processing_time_ms: u64,
}
```

### 6.3 Status Determination (Deterministic Rules)
Status is computed only inside **Iron Core** using fixed rules (SAFE / WARNING / CRITICAL).
UI renders color, nothing more.

---

## VII. IRON CORE ↔ DASHBOARD BOUNDARY

**Iron Core MUST:**
- Return valid `ProjectTruth` JSON
- Sort risks and actions deterministically
- Respond < 500ms for <100k rows

**Dashboard MUST NOT:**
- Calculate percentages
- Decide status
- Filter or re-rank data
- Cache stale truth

**UI = Glass Panel, not brain.**

---

## VIII. TESTING & ENFORCEMENT

### Required Tests
- Determinism test (byte equality)
- Contract validation tests
- Performance ceiling tests
- Memory ceiling tests

### Compile-Time Enforcement
- All constructors validate contracts before creation.

---

## IX. VERSIONING LAW

- **Semantic Versioning**
- No breaking change without **MAJOR** bump
- Old versions must remain readable

---

## X. FINAL DECLARATION

> **This document is the single constitution of Mission 017.**
> - **TableTruth** is foundational.
> - **ProjectTruth** is derived.
> - **UI** is obedient.
>
> Any code or document contradicting this contract is **invalid by definition**.

**Status:** APPROVED & LOCKED
