# MISSION: 017 — Deep Table Logic (Polars + Docling v2)

**Mission ID:** 017  
**Status:** PLANNING  
**Created:** 2026-01-31  
**IIP Version:** 1.2  
**Project Type:** ENGINE

---

## 1. DECISION CONTEXT

### Background
Mission 016 (Iron Hand Protocol) successfully cleaned the repository and established a stable V1.0 foundation. TachFileTo now has:
- Clean crate structure with `/vendor` organization
- Fortified `.gitignore` preventing pollution
- Organized documentation hierarchy (`/docs/specs` and `/docs/archive`)
- Stable `main` branch ready for advanced features

### Problem Statement
Current extraction capabilities (Docling v1) provide basic text and layout extraction, but **table processing remains shallow**:
1. Tables are extracted as raw structures without semantic understanding
2. No integration with modern data processing libraries (Polars)
3. Limited ability to transform extracted tables into queryable datasets
4. Missing validation for complex table structures (merged cells, nested headers)

### Strategic Importance
This mission establishes TachFileTo as a **data-first extraction engine**:
- Enables downstream analytics and BI integration
- Provides foundation for BOQ (Bill of Quantities) processing
- Demonstrates deterministic table parsing at scale
- Positions TachFileTo for enterprise document processing workflows

---

## 2. SCOPE

### In-Scope
1. **Polars Integration**
   - Add Polars as dependency for table processing
   - Create conversion layer: Docling tables → Polars DataFrames
   - Implement schema inference for extracted tables
   - Add basic table validation (column count, data types)

2. **Docling v2 Upgrade**
   - Upgrade from Docling v1 to v2 (if available)
   - Leverage enhanced table detection capabilities
   - Integrate improved cell merging logic
   - Utilize advanced layout analysis

3. **Table Processing Pipeline**
   - Design deterministic table extraction workflow
   - Implement table confidence scoring
   - Add table metadata enrichment (row/column counts, headers)
   - Create export formats (CSV, Parquet, JSON)

### Out-of-Scope (Explicitly Deferred)
- ❌ BOQ-specific business logic (Mission 018+)
- ❌ MasterFormat classification (Constitutional violation)
- ❌ Multi-document table aggregation (Phase 2)
- ❌ Machine learning-based table detection (Future research)
- ❌ Real-time streaming table processing (Performance optimization phase)

### Files to Modify
- `libs/elite_pdf/Cargo.toml` (ADD Polars dependency)
- `libs/elite_pdf/src/table_processor.rs` (NEW)
- `libs/elite_pdf/src/polars_bridge.rs` (NEW)
- `libs/elite_pdf/build.rs` (VERIFY compatibility)
- `src/core/extraction_engine.rs` (MODIFY - integrate table pipeline)

### Files NOT to Touch
- `docs/BOUNDARY_MANIFEST.md` (Constitutional - frozen)
- `.gitignore` (Just fortified in Mission 016)
- `libs/elite_pdf/vendor/` (Stable library location)

---

## 3. TASKS

### 3.1 Research & Planning
- [ ] Evaluate Polars vs Arrow for table processing
- [ ] Review Docling v2 changelog and migration guide
- [ ] Design table extraction pipeline architecture
- [ ] Define table confidence scoring algorithm

### 3.2 Dependency Integration
- [ ] Add Polars to `Cargo.toml` with appropriate features
- [ ] Verify build stability with new dependencies
- [ ] Update `build.rs` if needed for Polars linking
- [ ] Test compatibility with existing MuPDF integration

### 3.3 Core Implementation
- [ ] Create `table_processor.rs` module
- [ ] Implement Docling → Polars conversion
- [ ] Add table validation logic
- [ ] Create export utilities (CSV, Parquet)

### 3.4 Testing & Validation
- [ ] Unit tests for table conversion
- [ ] Integration tests with real PDF samples
- [ ] Performance benchmarks (tables/second)
- [ ] Determinism verification (same input → same output)

---

## 4. ACCEPTANCE CRITERIA

### 4.1 Functional Requirements
- [ ] Extract tables from PDF with \u003e 90% structural accuracy
- [ ] Convert extracted tables to Polars DataFrames
- [ ] Export tables to CSV/Parquet with correct schema
- [ ] Handle merged cells and complex headers

### 4.2 Constitutional Compliance
- [ ] **Zero Business Logic:** No BOQ interpretation, no cost estimation
- [ ] **Deterministic:** Same PDF → same table output (byte-identical)
- [ ] **Contract-Only:** All exports conform to defined schemas
- [ ] **Fail-Fast:** Invalid tables rejected with clear error messages

### 4.3 Performance
- [ ] Table extraction \u003c 2 seconds per page (average)
- [ ] Memory usage \u003c 500MB for 100-page documents
- [ ] Build time increase \u003c 30 seconds (Polars compilation)

### 4.4 Documentation
- [ ] Update `README.md` with table processing capabilities
- [ ] Document Polars integration in `/docs/specs`
- [ ] Add table extraction examples
- [ ] Create migration guide from v1 to v2

---

## 5. RISKS & CONSTRAINTS

### 5.1 Technical Risks
- **Risk:** Polars dependency increases binary size significantly
  - **Mitigation:** Use feature flags to make Polars optional

- **Risk:** Docling v2 may have breaking API changes
  - **Mitigation:** Maintain compatibility layer for v1

### 5.2 Scope Creep Risks
- **Risk:** Temptation to add BOQ-specific logic
  - **Mitigation:** Strict constitutional review

- **Risk:** Adding "smart" table interpretation
  - **Mitigation:** Keep processing purely structural

### 5.3 Dependencies
- **Dependency:** Stable Rust toolchain (Edition 2024)
  - **Status:** Already configured

- **Dependency:** Clean repository (Mission 016)
  - **Status:** ✅ Complete

---

## 6. SUCCESS METRICS

**Mission succeeds if:**
1. Tables extracted from PDF can be queried with Polars
2. Export formats (CSV/Parquet) are valid and deterministic
3. No constitutional violations detected
4. Build remains stable with new dependencies

**Mission fails if:**
1. Table extraction accuracy \u003c 80%
2. Memory usage exceeds 1GB for typical documents
3. Build time increases \u003e 2 minutes
4. Business logic leaks into extraction layer

---

**Next Step:** Research Polars integration patterns  
**Estimated Duration:** 4-6 hours (implementation) + 2 hours (testing)  
**Blocking Issues:** None (Mission 016 complete)
