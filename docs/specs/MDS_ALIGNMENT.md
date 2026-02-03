# MDS ALIGNMENT (MASTER INDEX)

> **PRIME DIRECTIVE:** 
> **"Làm sao để một kỹ sư 50 tuổi, sau 8 tiếng làm việc, nhìn vào màn hình này và dám ký tên vào báo cáo?"**
> *(How to make a 50yo engineer, after 8 hours of work, look at this screen and dare to sign the report?)*

**Status:** ACTIVE & CENTRAL
**Purpose:** Central registry for the "Four Pillars" of TachFileTo Forensic Workstation.

---

## 🏛️ THE FOUR PILLARS (CORE SPECS)

These 4 documents constitute the **complete** law of the system.

### 1. 🛡️ TRUTH CONTRACT (The Steel Core)
- **File:** [CORE_1_TRUTH_CONTRACT.md](./CORE_1_TRUTH_CONTRACT.md)
- **Role:** Defines structural truth, rejections, and "Zero Business Logic" boundary.
- **Key Concept:** `TableTruth` and `ProjectTruth` structs.

### 2. ⚡ DATA CONSISTENCY LOGIC (The Technical Truth)
- **File:** [CORE_2_DATA_LOGIC.md](./CORE_2_DATA_LOGIC.md)
- **Role:** Ensures Arithmetic Integrity, Provenance, and Cross-Source Consistency.
- **Key Concept:** Rules R01-R05 (Math, Lineage, Encoding, Anomaly, Mismatch).

### 3. 👁️ UI CONSTITUTION (The Visual Interface)
- **File:** [CORE_3_UI_CONSTITUTION.md](./CORE_3_UI_CONSTITUTION.md)
- **Role:** Defines the Forensic Workstation UI (Brutalist, High Contrast, No Shadows).
- **Key Concept:** "Truth over UX", 4-Panel Architecture.

### 4. 🧠 HUMAN AUDIT PROTOCOL (The Human Gate)
- **File:** [CORE_4_HUMAN_AUDIT.md](./CORE_4_HUMAN_AUDIT.md)
- **Role:** Defines how humans intervene when the machine fails. Encoding Repair & Audit Logs.
- **Key Concept:** `Ledger of Corrections`, The Seal.

---

## 🔧 TECHNICAL APPENDIX (IMPLEMENTATION DETAILS)

Reference materials for developers. DO NOT override the Core Pillars.

- **[IPC_PROTOCOL.md](./technical/IPC_PROTOCOL.md)**: Stdio/Arrow hybrid communication.
- **[INGESTION_SCHEMA.json](./technical/INGESTION_SCHEMA.json)**: JSON schema for incoming data.
- **[ARCHITECTURE_BOUNDARY.md](./technical/ARCHITECTURE_BOUNDARY.md)**: System boundaries (Stateless vs Persistent).
- **[LATENCY_BUDGET.md](./technical/LATENCY_BUDGET.md)**: Performance constraints.
- **[DEV_GUIDE.md](../DEV_GUIDE.md)**: Build & FFI setup.

---

## 🗄️ ARCHIVE & LEGACY

- **[legacy_v1/](./legacy_v1/)**: Old concepts (Dashboard V3.0, etc.) - FOR REFERENCE ONLY.

---

## ✅ ALIGNMENT MANDATE
All new code must cite one of the **Four Pillars** as authority.
Any feature not supported by these 4 pillars is considered **Scope Creep** and must be rejected.
