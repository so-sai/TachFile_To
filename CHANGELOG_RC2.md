# CHANGELOG: TachFileTo V1.0.1-RC2
**Codename:** `EPSILON_SHIELD`
**Status:** SEALED & VERIFIED

## 🛡️ THE JUDGE'S VERDICT (R01-R05)
This release marks the transition of TachFileTo from a data extraction tool to a **Forensic Verification Workstation**. The core engine now enforces deterministic consistency laws.

### [R01] Internal Arithmetic Integrity
- **Logic:** Internal sum validation (e.g., Subtotal + VAT = Total).
- **The Law of Epsilon:** Mechanical enforcement of **1.0 VND** tolerance to filter floating-point noise while catching material errors.
- **Verification:** 100% Pass on `test_validation_r01`.

### [R05] Cross-Source Inconsistency Detection
- **Logic:** Cell-by-cell comparison between paired data sources (e.g., PDF Scan vs. Excel Workpaper).
- **Function:** Detects tampering or discrepancies between printed documents and electronic records.
- **Verification:** 100% Pass on `test_validation_r05`.

## 🏛️ CORE UPGRADES
- **Truth Export:** High-fidelity Markdown and Excel exporters with embedded metadata.
- **Encoding Gatekeeper:** Mission 021/022 integrated to prevent Mojibake injection.
- **Brutalist UI:** Space Grotesk font + 0px borders + enterprise-grade scrollbars for zero-cognitive-load auditing.
- **System Constitution:** Migrated all logic to 4-Pillar Spec Architecture (CORE 1-4).

## 📊 TECHNICAL STATS
- **Backend:** Rust / Polars (Nogil-Ready).
- **Latency Target:** < 200ms per 1000 rows.
- **ID Standard:** Deterministic Global Cell IDs ({table_id}_{row}_{col}).

---
*Seal authorized by Antigravity Intelligence.*
