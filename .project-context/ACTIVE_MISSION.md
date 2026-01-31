# MISSION: 2026-001 — UI DropZone & Ingestion Validation

**Mission ID:** 2026-001  
**Status:** EXECUTION (Phase 2 Extraction Engine Complete)  
**Created:** 2026-01-10  
**IIP Version:** 1.1  
**Project Type:** APP

---

## 1. DECISION CONTEXT

### Background
TachFileTo has established constitutional boundaries (`BOUNDARY_MANIFEST.md`) and a data contract (`INGESTION_SCHEMA.json` v0.1). However, there is currently **no user interface** to:
- Accept file drag-and-drop
- Validate files against schema
- Export signed JSON objects to `/temp/ingestion/`

### Problem Statement
Users cannot interact with TachFileTo because there is no UI implementation. We need a **minimal, stateless UI** that:
1. Accepts PDF/DOCX/XLSX files via drag-and-drop
2. Validates file structure (without processing content yet)
3. Generates mock `INGESTION_SCHEMA.json` v0.1 compliant objects
4. Displays validation results in < 0.1 seconds (Cockpit UI standard)

### Strategic Importance
This is the **first user-facing feature** of TachFileTo. It must:
- Demonstrate constitutional compliance (Stateless, Zero Business Logic)
- Prove schema validation works
- Establish UI performance baseline (< 0.1s response)
- Serve as foundation for future extraction engines

---

## 2. SCOPE

### In-Scope
1. **UI Components**
   - DropZone component (drag-and-drop area)
   - File validation indicator (visual feedback)
   - Export button (write to `/temp/ingestion/`)

2. **Validation Logic (Rust)**
   - File type detection (PDF/DOCX/XLSX)
   - File size limits (prevent memory overflow)
   - Schema structure validation (no content extraction yet)
   - Mock data generation for `pages` array

3. **Schema Compliance**
   - Generate valid `INGESTION_SCHEMA.json` v0.1 objects
   - Include all required fields (`source`, `document_type`, `checksum`, etc.)
   - Mock `origin_signature` (Ed25519 implementation deferred to Phase 2)
   - Mock `extraction_meta` with placeholder values

### Out-of-Scope (Explicitly Deferred)
- ❌ **JsonPreview & Clipboard** (Deferred to Phase 2 per Skeptic)
- [x] Actual PDF/DOCX content extraction (Implemented v2.0.0 via Docling)
- ❌ Ed25519 signature generation (Phase 4)
- [x] Real checksum calculation (Implemented SHA-256)
- ❌ iron_core integration (Phase 3)
- ❌ TTL cleanup mechanism (Phase 3)
- ❌ Multi-file batch processing (Phase 4)

### Files to Modify
- `ui/src/components/DropZone.tsx` (NEW)
- `ui/src/components/ValidationPanel.tsx` (NEW)
- `ui/src/components/JsonPreview.tsx` (NEW)
- `ui/src-tauri/src/commands/validate_file.rs` (NEW)
- `ui/src-tauri/src/lib.rs` (MODIFY - register commands)

### Files NOT to Touch
- `docs/BOUNDARY_MANIFEST.md` (Constitutional - frozen)
- `docs/specs/INGESTION_SCHEMA.json` (Contract - frozen)
- `.project-context/ANTI_GRAVITY.md` (Philosophy - frozen)
- Any `iron_core` related files

---

## 3. TASKS

### 3.1 Rust Backend (Validation Logic)
- [ ] Create `src-tauri/src/commands/validate_file.rs`
  - [ ] Implement `validate_file_type(path: String) -> Result<FileType, String>`
  - [ ] Implement `generate_mock_ingestion_object(path: String) -> Result<IngestionObject, String>`
  - [ ] Add file size limit check (max 100MB for Phase 1)

- [ ] Update `src-tauri/src/lib.rs`
  - [ ] Register `validate_file` command
  - [ ] Register `generate_mock_ingestion_object` command

- [ ] Create `src-tauri/src/models/ingestion.rs`
  - [ ] Define `IngestionObject` struct matching schema v0.1
  - [ ] Implement `serde` serialization
  - [ ] Add mock data generators

### 3.2 React Frontend (UI Components)
- [ ] Create `ui/src/components/DropZone.tsx`
  - [ ] Implement drag-and-drop handlers
  - [ ] Call Tauri `validate_file` command
  - [ ] Display validation status (< 0.1s requirement)

- [ ] Create `ui/src/components/ValidationPanel.tsx`
  - [ ] Show file metadata (name, size, type)
  - [ ] Display validation warnings (if any)
  - [ ] Show confidence scores (mocked for now)

- [ ] **DEFERRED:** `ui/src/components/JsonPreview.tsx` (Phase 2)

- [ ] Update `ui/src/App.tsx`
  - [ ] Integrate DropZone component
  - [ ] Wire up state management (Zustand)
  - [ ] Add export button (write to `/temp/ingestion/`)

### 3.3 Testing
- [ ] Rust unit tests
  - [ ] Test file type detection
  - [ ] Test mock object generation
  - [ ] Test file size limits

- [ ] React component tests (optional for Phase 1)
  - [ ] DropZone drag-and-drop behavior
  - [ ] Validation panel rendering

- [ ] Manual UI testing
  - [ ] Drag PDF → see validation result < 0.1s
  - [ ] Drag DOCX → see validation result < 0.1s
  - [ ] Drag invalid file → see error message
  - [ ] Export JSON → verify file written to `/temp/ingestion/`

---

## 4. ACCEPTANCE CRITERIA

### 4.1 Functional Requirements
- [ ] User can drag-and-drop PDF/DOCX/XLSX files
- [ ] Validation result appears in < 0.1 seconds
- [ ] Generated JSON matches `INGESTION_SCHEMA.json` v0.1
- [ ] Export button writes file to `/temp/ingestion/`
- [ ] Invalid files show clear error messages

### 4.2 Constitutional Compliance
- [ ] **Stateless:** No file paths stored in UI preferences
- [ ] **Zero Business Logic:** No MasterFormat assignment, no BOQ generation
- [ ] **Contract-Only:** All exports conform to schema v0.1
- [ ] **Fail-Fast:** Invalid files rejected immediately with clear errors

### 4.3 Performance
- [ ] Validation response < 0.1 seconds (Cockpit UI standard)
- [ ] UI remains responsive during file processing
- [ ] No memory leaks (test with 10 consecutive file drops)

### 4.4 Documentation
- [ ] Update `README_DEV.md` with UI testing instructions
- [ ] Add comments explaining mock data generation
- [ ] Document deferred features (Ed25519, real extraction)

---

## 5. RISKS & CONSTRAINTS

### 5.1 Technical Risks
- **Risk:** Tauri IPC overhead might exceed 0.1s budget
  - **Mitigation:** Use async commands, measure with `console.time()`

- **Risk:** Mock data might not match real extraction output
  - **Mitigation:** Design mocks based on actual PDF structure research

### 5.2 Scope Creep Risks
- **Risk:** Temptation to add "real" PDF extraction
  - **Mitigation:** Skeptic review will catch this

- **Risk:** Adding "helpful" features like recent files
  - **Mitigation:** Constitutional violation - auto-reject

### 5.3 Dependencies
- **Dependency:** `INGESTION_SCHEMA.json` v0.1 must be stable
  - **Status:** Frozen (committed in previous mission)

- **Dependency:** Tauri v2 capabilities must allow file access
  - **Status:** Already configured (`core:default`)

---

## 6. SUCCESS METRICS

**Mission succeeds if:**
1. User can drag-and-drop a file and see validation in < 0.1s
2. Generated JSON passes online schema validator
3. No constitutional violations detected by Skeptic
4. Code passes `cargo test` and TypeScript compilation

**Mission fails if:**
1. Validation takes > 0.1 seconds
2. Generated JSON doesn't match schema v0.1
3. Skeptic finds boundary violations
4. Code introduces state persistence

---

**Next Step:** AUDITING (Skeptic review)  
**Estimated Duration:** 2-3 hours (coding) + 30 minutes (testing)  
**Blocking Issues:** None
