# ANTI_GRAVITY.md — Core Philosophy

**Project:** TachFileTo  
**Version:** 3.1.1  
**IIP Protocol:** v1.1  
**Last Updated:** 2026-01-10

---

## I. CORE IDENTITY

> **"TachFileTo is intelligent at cleaning data, and deliberately ignorant at making decisions."**

### What TachFileTo IS
- **Data Ingestion Organ** in the Elite 10 ecosystem
- **Stateless Pre-Processor** for construction documents
- **Schema Validator** enforcing `INGESTION_SCHEMA.json` v0.1
- **Cryptographic Signer** proving data authenticity

### What TachFileTo IS NOT
- ❌ Business logic engine
- ❌ Database system
- ❌ Decision-making authority
- ❌ Standalone application

---

## II. CONSTITUTIONAL PRINCIPLES

### 1. Stateless by Design
**Doctrine:** No persistence beyond runtime memory (except UI preferences).

**Rationale:**
- Prevents context leakage between projects
- Eliminates "hidden state" bugs
- Forces explicit data contracts
- Enables true isolation

**Enforcement:**
- 5-minute TTL on all ingestion objects
- `.gitignore` blocks database commits
- UI preferences MUST NOT contain project metadata

---

### 2. Least Privilege Architecture
**Doctrine:** TachFileTo has minimal capabilities, iron_core has absolute authority.

**Rationale:**
- Reduces attack surface
- Prevents scope creep
- Maintains clear separation of concerns
- Protects SSOT (Single Source of Truth)

**Enforcement:**
- Tauri capabilities locked to `core:default`
- Filesystem access limited to `/temp/ingestion/` and user-selected files
- No network access
- No shell execution

---

### 3. Contract-Only Communication
**Doctrine:** All data exchange via `INGESTION_SCHEMA.json`, no direct function calls.

**Rationale:**
- Forces explicit API definition
- Enables version compatibility checks
- Prevents tight coupling
- Allows independent evolution

**Enforcement:**
- Schema validation before export
- Origin signature (Ed25519) on every object
- iron_core has veto power (can reject valid schemas)

---

### 4. Zero Business Logic
**Doctrine:** TachFileTo normalizes data, iron_core interprets it.

**Rationale:**
- Prevents duplicate logic across ecosystem
- Ensures SSOT for business rules
- Simplifies testing (no mocking needed)
- Reduces maintenance burden

**Forbidden Operations:**
- ❌ MasterFormat code assignment
- ❌ BOQ generation
- ❌ Price calculations
- ❌ Timeline inferences
- ❌ Quantity modifications

---

### 5. Fail-Fast Philosophy
**Doctrine:** Report errors immediately, never silently recover.

**Rationale:**
- Construction data errors are expensive
- Silent failures corrupt SSOT
- Users need immediate feedback
- Debugging is easier with explicit failures

**Implementation:**
- Rust `Result<T, E>` types (no `unwrap()`)
- UI validation reports (non-blocking alerts)
- Schema validation before export
- Checksum verification

---

## III. ECOSYSTEM POSITION

### Upstream Dependencies
```
Raw Construction Documents
  ↓
User Drag-and-Drop
  ↓
[TachFileTo] ← YOU ARE HERE
```

**Constraints:**
- No assumptions about file quality
- No assumptions about naming conventions
- No assumptions about project context

---

### Downstream Consumers
```
[TachFileTo]
  ↓
Signed Ingestion JSON
  ↓
[iron_core SSOT]
  ↓
Business Logic & Persistence
```

**Guarantees:**
- Schema-valid JSON
- Cryptographic authenticity
- Integrity checksums
- Forensic metadata

---

## IV. ANTI-PATTERNS (FORBIDDEN)

### 🚫 The "Helpful Assistant" Anti-Pattern
**Description:** Adding "convenience features" that violate boundaries.

**Examples:**
- "Let me auto-assign MasterFormat codes for you"
- "I'll save your recent projects"
- "I can calculate totals while I'm at it"

**Why Forbidden:**
- Violates Stateless principle
- Duplicates iron_core logic
- Creates context leakage
- Breaks SSOT

---

### 🚫 The "Smart Cache" Anti-Pattern
**Description:** Storing processed results to "improve performance".

**Examples:**
- Caching extracted tables
- Remembering previous validations
- Storing file hashes across sessions

**Why Forbidden:**
- Violates Stateless principle
- Creates stale data risks
- Complicates debugging
- Breaks isolation

---

### 🚫 The "Direct Access" Anti-Pattern
**Description:** Reading iron_core files directly instead of using contracts.

**Examples:**
- Reading iron_core's SQLite database
- Importing iron_core Rust modules
- Sharing data structures

**Why Forbidden:**
- Violates Contract-Only principle
- Creates tight coupling
- Prevents independent evolution
- Breaks architectural boundaries

---

## V. DESIGN HEURISTICS

### When Adding a Feature, Ask:
1. **Does this violate Stateless?**
   - Will it persist data beyond runtime?
   - Will it remember previous sessions?

2. **Does this violate Least Privilege?**
   - Does it need new Tauri capabilities?
   - Does it access files outside scope?

3. **Does this violate Zero Business Logic?**
   - Does it interpret data meaning?
   - Does it make decisions for the user?

4. **Does this violate Contract-Only?**
   - Does it bypass `INGESTION_SCHEMA.json`?
   - Does it directly call iron_core?

**If ANY answer is YES → REJECT the feature.**

---

## VI. EVOLUTION CONSTRAINTS

### Allowed Changes
- ✅ New extraction engines (PDF, OCR, etc.)
- ✅ Better normalization algorithms
- ✅ Improved UI/UX (within boundaries)
- ✅ Performance optimizations (stateless)

### Forbidden Changes
- ❌ Adding persistence layer
- ❌ Implementing business rules
- ❌ Expanding Tauri capabilities without review
- ❌ Breaking schema compatibility

---

## VII. MISSION STATEMENT

> **TachFileTo exists to transform chaotic construction documents into clean, validated, cryptographically-signed JSON objects that iron_core can trust absolutely.**

We succeed when:
- Users can drag-and-drop any file format
- Validation happens in < 0.1 seconds
- iron_core never receives invalid data
- No project data leaks between sessions

We fail when:
- We try to be "smart" about business logic
- We cache data for "convenience"
- We bypass contracts for "performance"
- We violate boundaries for "features"

---

**This document is the gravitational force that keeps TachFileTo in its proper orbit.**  
**Violating these principles is not a bug—it's an architectural failure.**

---

## VIII. TECHNOLOGY-SPECIFIC CONSTRAINTS

### Rust Backend (Iron Core V3.0)

**Edition & Toolchain:**
- **Rust Edition:** 2024 (bleeding edge stability)
- **Minimum Version:** 1.92.0
- **Rationale:** Future-proofing with latest safety and efficiency features

**Data Processing Stack:**
- **Polars:** 0.52 (DataFrame engine for 1M+ rows)
  - Use `.into()` to convert `Series` → `Column`
  - Enable lazy evaluation for large datasets
- **Calamine:** 0.32 (Excel parser)
  - **MUST** enable `dates` feature in `Cargo.toml`
  - Use `open_workbook_auto()` for universal .xls/.xlsx support

**Smart Header Detection (Iron Core V3.0):**
- **Fuzzy Matching:** Jaro-Winkler threshold ≥ 0.85 for Vietnamese QS terms
- **Merged Cell Propagation:** Hierarchical headers (e.g., "Khối lượng" → "Kỳ trước/Kỳ này/Lũy kế")
- **Metadata Skipping:** Scan rows 0-50, detect header via keyword density
- **Numeric Penalty:** -0.5 score for rows with >70% numbers (likely data, not headers)
- **Footer Filtering:** Auto-ignore rows with ["Tổng cộng", "Cộng", "Ký tên", "Ghi chú", "Xác nhận"]

**Error Handling:**
- **Mandatory:** Use `Result<T, E>` for all fallible operations
- **Forbidden:** `unwrap()`, `expect()` without clear justification
- **Preferred:** `anyhow` for application errors, custom types for domain errors

---

### React Frontend (Cockpit UI)

**Framework & Language:**
- **React:** 19.x (latest stable)
- **TypeScript:** ~5.8.3 (strict mode enabled)
- **Build Tool:** Vite 7.x
- **State Management:** Zustand 5.x (minimal, predictable)

**Design Language:**
- **Style:** Brutalist/Enterprise (hard edges, zero ambiguity)
- **Colors:** `#DC2626` (Red/ĐỎ), `#059669` (Green/XANH), `#F59E0B` (Yellow/VÀNG)
- **Typography:** Monospace for numbers, Sans-serif for labels
- **Density:** Enterprise-grade (32px row height, compact spacing)

**Performance Requirements:**
- **Virtual Scrolling:** Mandatory for 1M+ rows (TanStack Virtual)
- **Response Time:** ≤ 100ms for all interactions (see `UI_LATENCY_CONTRACT.md`)
- **Memory:** < 500MB for 1M rows
- **Frame Rate:** 60fps during scrolling

**Language:**
- **UI Labels:** 100% Vietnamese (XANH/VÀNG/ĐỎ status)
- **Error Messages:** Vietnamese with technical details in English (if needed)
- **Abbreviations:** QS-standard (ĐG = Đơn giá, KL = Khối lượng, T.TIỀN = Thành tiền)

---

### Status Determination Rules (Business Logic)

**Critical (ĐỎ):**
- Deviation ≥ 15% **OR**
- Risk count ≥ 5 **OR**
- Profit margin ≤ 0%

**Safe (XANH):**
- Deviation < 5% **AND**
- Risk count == 0 **AND**
- Profit margin > 10%

**Warning (VÀNG):**
- Everything else

**Rationale:** Financial health (profit) overrides operational metrics (deviation).

---

### Forbidden Technologies & Patterns

**Absolutely Forbidden:**
- ❌ **Python:** Exterminated in V2.3 (backend/ folder removed)
- ❌ **STDIO IPC:** Use Tauri Commands only
- ❌ **English Status Labels:** Must use XANH/VÀNG/ĐỎ
- ❌ **Virtual Environments:** No venv, pip, or Python tooling
- ❌ **Legacy Font Conversion:** TCVN3/VNI deferred to V2.6
- ❌ **Cloud Sync/SaaS:** Desktop-first, offline-only architecture
- ❌ **AI/ML Inference:** Deterministic algorithms only

**Rationale:** These technologies either:
1. Violate architectural principles (Python = non-deterministic)
2. Create deployment complexity (Cloud = requires auth)
3. Compromise performance (Legacy fonts = slow)

---

### Development Workflow

**Running Application:**
```bash
cd ui
npm run tauri dev
# Expected: Vite dev server on port 1420, Tauri window launches
```

**Testing:**
```bash
cd ui/src-tauri
cargo test --lib
# Expected: 33/33 tests PASSING (Iron Core V3.0 validated)
```

**Building Production:**
```bash
cd ui
npm run tauri build
# Output: ui/src-tauri/target/release/tachfileto-core.exe
```

---

### Single-Thread Enforcement

**Rule:** Never open multiple browser tabs or duplicate processes if target service is not confirmed "READY".

**Rationale:** Prevents AI agent hallucination loops (Tab Overflow Incident - see `LESSONS.md` #8)

**Implementation:**
1. Verify terminal command exit code before proceeding
2. Check port availability before browser interaction
3. Stop immediately if environment state is ambiguous

---

**These constraints are non-negotiable. Violating them requires Human Architect approval.**


## IX. ECOSYSTEM CONSTITUTION (IRON CORE)

> **“iron_core là trung tâm logic nghiệp vụ của hệ sinh thái. Không app nào được sử dụng logic của iron_core nếu chưa có Contract được Architect phê duyệt.”**

**Implementation via `BOUNDARY_MANIFEST.md`:**
1.  **Restricted Access:** `iron_core` is `CENTRAL_LOGIC — RESTRICTED`.
2.  **Embassy Pattern:** All data exchange MUST use `core_contract` structs.
3.  **Logic Sovereignty:** No business logic (Price, Diff, Ledger) in local app code.

---

## X. MDS ALIGNMENT (MDS-ELITE10-2026-v1.0)

> **TachFileTo = OPTIONAL Heavy Ingestion Tool**

### Classification per MDS §II.3
- **Role:** Heavy PDF/OCR processing (khi cần)
- **Status:** `OPTIONAL` – Không bắt buộc cho hệ sinh thái
- **Dependency:** AutoQSVN/iron_core KHÔNG phụ thuộc TachFileTo

### LAW Compliance
| Law | Requirement | Status |
|-----|-------------|--------|
| LAW-03 | Payment Safety | ✅ Không tham gia payment flow |
| LAW-04 | Optional Module Rule | ✅ Không trong critical path |

### Existence Conditions (MDS §II.3)
TachFileTo chỉ được triển khai khi có **cả 3 điều kiện**:
1. PDF nặng / scan
2. Cần OCR / AI
3. Chi phí xử lý cao

**Reference:** [docs/MDS_ALIGNMENT.md](../docs/MDS_ALIGNMENT.md)
