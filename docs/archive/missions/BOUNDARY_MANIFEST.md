# TachFileTo Boundary Manifest
**Version:** 1.1.0  
**MDS Reference:** MDS-ELITE10-2026-v1.0  
**Ecosystem Role:** `OPTIONAL` – Heavy Ingestion Tool  
**Status:** Constitutional (Non-Negotiable)  
**Last Updated:** 2026-01-12

---

## I. ORGAN IDENTITY

### Role in Elite 10 Ecosystem
**TachFileTo = Data Ingestion & Pre-Processing Organ**

> **Constitutional Doctrine:**  
> *"TachFileTo is intelligent at cleaning data, and deliberately ignorant at making decisions."*

### Position in Data Flow
```
RAW FILES (PDF/DOCX/XLSX/Scans)
        ↓
[TachFileTo] ← YOU ARE HERE
  - Split & Normalize
  - Clean & Validate Schema
  - Export Ingestion Objects
        ↓
VALIDATED JSON (Schema-Checked)
        ↓
[iron_core] ← SINGLE SOURCE OF TRUTH
  - Business Logic
  - MasterFormat Assignment
  - BOQ Generation
  - Persistence
```

### Upstream Dependencies
- Raw construction documents (any format)
- User drag-and-drop actions
- **NO** external APIs or cloud services

### Downstream Consumers
- **ONLY** `iron_core` (via `INGESTION_SCHEMA.json` contract)
- **NO** direct database writes
- **NO** file system writes outside designated temp directory

---

## II. ALLOWED OPERATIONS

### ✅ Data Processing (Core Competency)
1. **File Splitting**
   - Extract pages from multi-page PDFs
   - Separate tables from scanned documents
   - Detect document boundaries

2. **Normalization**
   - ASCII cleanup (Vietnamese diacritics handling)
   - Whitespace standardization
   - Date format unification (DD/MM/YYYY → ISO 8601)

3. **Schema Validation**
   - Validate against `INGESTION_SCHEMA.json`
   - Generate validation reports (non-blocking)
   - Confidence scoring (0.0 - 1.0)

4. **Metadata Extraction**
   - Page numbers
   - Content type detection (table/text/image)
   - Checksum generation (SHA-256)
   - **Origin signature** (cryptographic proof of TachFileTo authorship)

### ✅ UI Operations (User Experience)
1. **Preview & Visualization**
   - Read-only data preview
   - Table structure visualization
   - Confidence heatmaps

2. **Export**
   - Write to `/temp/ingestion/` directory ONLY
   - JSON format per `INGESTION_SCHEMA.json`
   - Automatic cleanup after 5 minutes (TTL enforcement)

3. **UI Preferences (RESTRICTED)**
   - Window size/position
   - Theme selection (light/dark)
   - **ABSOLUTELY FORBIDDEN:** Recent files list, project names, file paths

---

## III. FORBIDDEN OPERATIONS

### ❌ ABSOLUTELY FORBIDDEN (NON-NEGOTIABLE)

Under **no circumstances** shall TachFileTo:

1. **Business Logic Violations**
   - ❌ Infer, calculate, or modify quantities
   - ❌ Assign or suggest MasterFormat codes
   - ❌ Generate BOQ or contract summaries
   - ❌ Perform price calculations or adjustments
   - ❌ Make timeline or schedule inferences

2. **Data Persistence Violations**
   - ❌ Persist any project data beyond runtime memory
   - ❌ Create or modify databases (SQLite, PostgreSQL, etc.)
   - ❌ Write to iron_core's file system
   - ❌ Cache processed results across sessions

3. **Architectural Violations**
   - ❌ Clone or embed iron_core codebase
   - ❌ Directly call iron_core functions (must use JSON contract)
   - ❌ Read iron_core's database files
   - ❌ Bypass schema validation

4. **Security Violations**
   - ❌ Upload data to cloud services
   - ❌ Make external network requests
   - ❌ Store credentials or API keys
   - ❌ Log sensitive project information

5. **Context Leakage Violations**
   - ❌ Store project metadata in UI config files
   - ❌ Maintain "Recent Projects" history
   - ❌ Cross-reference data between projects
   - ❌ Retain file paths after processing

---

## IV. INTEGRATION CONTRACT

### Communication Protocol with iron_core

**Contract:** `docs/specs/INGESTION_SCHEMA.json` v0.1

#### TachFileTo Responsibilities
1. **Export** valid JSON objects to `/temp/ingestion/`
2. **Sign** each object with `origin_signature`
3. **Self-destruct** objects after 5-minute TTL
4. **Report** validation failures to user (UI only)

#### iron_core Responsibilities
1. **Poll** `/temp/ingestion/` directory
2. **Verify** `origin_signature` authenticity
3. **Validate** against schema
4. **Reject** invalid objects (return error code)
5. **Acknowledge** successful ingestion (trigger TachFileTo cleanup)

#### Rejection Handling
- iron_core has **absolute authority** to reject any object
- TachFileTo **MUST NOT** retry or modify rejected objects
- User must be notified via UI (non-blocking alert)

---

## V. SECURITY BOUNDARIES

### Filesystem Access Limits

**ALLOWED:**
- `/temp/ingestion/` (write-only, auto-cleanup)
- User-selected input files (read-only, one-time access)

**FORBIDDEN:**
- iron_core workspace directory
- System directories (Program Files, Windows, etc.)
- Other projects in ecosystem

### Tauri Capabilities (Locked)

**Current:** `capabilities/default.json`
```json
{
  "permissions": ["core:default"]
}
```

**Future Expansion (Phase 3 - PDF Processing):**
- `fs:read-file` (scoped to user-selected files only)
- `fs:write-file` (scoped to `/temp/ingestion/` only)

**NEVER ALLOWED:**
- `shell:execute`
- `http:request`
- `dialog:save` (outside temp directory)

### Cryptographic Guarantees

**Origin Signature (Ed25519)**
- Each ingestion object signed with TachFileTo instance private key
- iron_core verifies signature before processing
- Prevents manual JSON tampering or injection attacks

**Checksum (SHA-256)**
- File integrity verification
- Detects corruption during transfer
- Does NOT prove authorship (use `origin_signature` for that)

---

## VI. SELF-DESTRUCT MECHANISM

### Time-to-Live (TTL) Enforcement

**Policy:** All ingestion objects expire after **5 minutes**

**Implementation:**
1. TachFileTo writes JSON + timestamp to `/temp/ingestion/`
2. Background thread monitors directory every 30 seconds
3. Objects older than 5 minutes → **automatic deletion**
4. iron_core acknowledgment → **immediate deletion**

**Rationale:**
- Prevents data accumulation in temp storage
- Forces iron_core to consume data promptly
- Reduces attack surface for data exfiltration

---

## VII. ENFORCEMENT & COMPLIANCE

### Code Review Checklist
Before merging any TachFileTo code:
- [ ] No business logic (MasterFormat, BOQ, pricing)
- [ ] No database writes outside `/temp/ingestion/`
- [ ] No iron_core codebase clones or imports
- [ ] UI preferences contain zero project metadata
- [ ] All exports conform to `INGESTION_SCHEMA.json`

### AI Agent Guardrails
This manifest is referenced in `.project-context/PROJECT_PROMPT.md` to constrain AI behavior.

**AI agents MUST:**
- Refuse requests to add business logic
- Reject attempts to bypass schema validation
- Alert user when boundary violations are requested

---

## VIII. REVISION HISTORY

| Version | Date       | Changes                                    |
|---------|------------|--------------------------------------------|
| 1.0.0   | 2026-01-10 | Initial constitutional boundary definition |

---

**Signed:** Elite 10 Ecosystem Architecture Board  
**Authority:** Non-negotiable for all TachFileTo development
