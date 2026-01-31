# Mission 020: The Pre-Truth Janitor (Adapter Layer)

> **Status:** DRAFT (V0.1)
> **Layer:** Adapter (Pre-Truth)
> **Philosophy:** "Clean the mud before entering the house."

## 1. The Architectural Mandate

Mission 019 established a critical "Iron Truth" principle:
> **`TableTruth` must be immutable and strict. It cannot repair data.**

To handle the "dirty reality" of construction data (OCR errors, typos, ghost columns) without compromising the Truth Layer, we introduce a dedicated **Adapter Layer**.

### The Cognitive Stack
1.  **Sensor Layer (Docling)**: Senses the world. High uncertainty. (Output: Raw JSON)
2.  **Adapter Layer (The Janitor)**: Cleans, repairs, and standardizes. (Output: Canonical JSON)
3.  **Truth Layer (Iron Table)**: Validates strict contracts. Rejects failures. (Output: `TableTruth`)
4.  **Engine Layer (Iron Engine)**: Computes deterministic reality. (Output: `ProjectTruth`)

## 2. Responsibilities of The Janitor

The Janitor is the **ONLY** place where "Heuristic Repair" is allowed.

### 2.1. Structural Purge (The Ghost Ejector)
- **Problem**: OCR detects empty vertical lines as columns.
- **Rule**: If a column has `Confidence < 0.7` for **100%** of its cells, AND implies no semantic value, DROP the column entirely.
- **Constraint**: Must log the purge event.

### 2.2. Syntax Repair (The Unit Stripper)
- **Problem**: Docling reads `1.250,50 m3` as Text. Schema expects Float.
- **Rule**:
    - Identify target column type (e.g., Float).
    - Regex strip non-numeric suffix/prefix (units).
    - Normalize decimal separators (VN `,` vs US `.`).
    - **Constraint**: If parsing fails after stripping -> Leave as Text (let Truth Layer reject it).

### 2.3. Encoding Normalization (The Translator)
- **Problem**: Mixed TCVN3, VNI, Unicode.
- **Rule**: Detect legacy charsets and normalize to NFKC Unicode **before** any other processing.

## 3. Implementation Contract

### Interface
```rust
pub trait Janitor {
    /// Takes raw Docling output and returns a "Sanitized" JSON Value
    /// ready for TableTruth deserialization.
    fn clean(raw: serde_json::Value) -> Result<serde_json::Value, JanitorError>;
}
```

### Prohibition (Legacy Debt Prevention)
- The Janitor **MUST NOT** infer business logic (e.g., "If column is missing, assume 0").
- The Janitor **MUST NOT** hallucinate data (e.g., filling gaps).
- The Janitor **ONLY** fixes syntactic and structural noise.
- **CRITICAL**: The Janitor **MUST NOT** increase confidence scores. Confidence is an epistemic property assigned by the Sensor and verified by the Truth. The Janitor must preserve or decrease confidence if it modifies data, but never boost it.

## 4. Migration from Mission 019
The experimental `TableTruth::sanitize()` method implemented in Mission 019 must be:
1.  **Refactored** out of `iron_table`.
2.  **Moved** to a new crate `iron_janitor` (or `iron_adapter`).
3.  **Expanded** to cover the full scope of this spec.
