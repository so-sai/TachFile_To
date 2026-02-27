Aligned with: [MDS_ALIGNMENT.md]

# CORE 4: HUMAN AUDIT PROTOCOL
## THE "HUMAN-IN-THE-LOOP" CONSTITUTION

**Status:** ACTIVE & ENFORCED
**Scope:** Encoding Repair, Data Correction, and Audit Logging.

> **Prime Directive:** "When the machine is unsure, the human decides. Every decision must be signed, recorded, and irreversible."

---

## 1. THE PHILOSOPHY OF INTERVENTION

We accept that raw data (PDF/Excel from AEC industry) is **inherently dirty**.
- Mojibake (Encoding errors) is common.
- Merged cells are common.
- Typos are common.

**The Iron Rule:** Use the machine to **isolate** the problem, but use the human to **fix** it.
Never let the machine "guess" a fix without a trail.

---

## 2. THE JANITOR (NGƯỜI GÁC CỔNG)

The **Janitor** (iron_adapter) is the first line of defense.
It sits between the Raw Ingestion (Docling/Calamine) and the Iron Core (Truth).

### 2.1 Responsibilities
1.  **Block Fatal Corruption**: Usage of `EncodingGatekeeper` to reject files with >1% unknown characters.
2.  **Flag Suspicious Data**: Identify potential Mojibake (e.g., "ThÃ©p", "Khá»‘i").
3.  **Propose Repairs**: Suggest "Windows-1258 to UTF-8" conversion, but **DO NOT APPLY** without user confirmation (unless confidence is 100% via heuristic dictionary match).

### 2.2 Rejection Policy
- If `EncodingError` > Critical Threshold → **HARD REJECT**.
- If `StructureError` (Merged Headers) → **HARD REJECT**.
- If `Typos` → **SOFT WARNING** (Allow human to fix).

---

## 3. THE REPAIR LOOP (HUMAN GATE)

When a file is "Tainted" (Vấn đề), it enters the Repair Loop in the UI.

### 3.1 The 4-Step Repair Protocol
1.  **Isolate**: The UI highlights ONLY the specific cell/row in question.
2.  **Evidence**: Panel A shows the PDF crop of that specific cell.
3.  **Verdict**: The user selects:
    - [ ] **Accept Original** (It's correct, machine was wrong)
    - [ ] **Apply Fix** (Use suggested encoding/value)
    - [ ] **Manual Override** (Type new value)
    - [ ] **Reject Row** (Ignore this line item)
4.  **Seal**: The decision is saved to the `Ledger of Corrections`.

---

## 4. THE LEDGER OF CORRECTIONS (THE AUDIT TRAIL)

Every manual intervention creates an immutable record.

### 4.1 Ledger Schema
```rust
struct CorrectionEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub actor_id: String,           // Local User / Engineer ID
    pub target_ref: GlobalId,       // {table_id}::{row}::{col}
    pub operation: OperationType,   // REPAIR_ENCODING | OVERRIDE_VALUE | IGNORE_ROW
    pub before: String,
    pub after: String,
    pub evidence_hash: String,      // Hash of the visual crop used as proof
    pub reason_code: String,        // "MOJIBAKE_DETECTED", "OCR_ERROR"
}
```

### 4.2 The "Seal" (Niêm Phong)
- Before the final report (ProjectTruth) is generated, the Ledger must be **Sealed**.
- A Sealed Ledger generates a SHA-256 hash.
- This hash is printed on the final PDF/Markdown report.
- **Result:** "If this report is altered, the hash won't match."

---

## 5. UI IMPLICATIONS

1.  **No "silent fixes"**: If the system auto-corrects encoding, it MUST show an "Auto-Repaired" badge.
2.  **One-click Audit**: Clicking a value must show its history (Raw -> Janitor -> Human -> Final).
3.  **The "Engineer's Signature"**: The final Seal action is equivalent to a physical signature. It requires explicit confirmation.

---

**END OF PROTOCOL**
