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
