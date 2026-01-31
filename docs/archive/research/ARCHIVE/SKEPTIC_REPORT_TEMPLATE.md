# SKEPTIC_REPORT.md — Template

**Mission ID:** [To be filled]  
**Mission Title:** [To be filled]  
**Skeptic Agent:** AGENT S  
**Report Date:** [To be filled]  
**IIP Version:** 1.1

---

## I. EXECUTIVE SUMMARY

**Verdict:** `PASS` | `WARN` | `FAIL`

**One-line Assessment:**
[Brief statement of overall risk level]

---

## II. NEGATIVE ANALYSIS

### 2.1 Boundary Violations Risk
**Question:** Does this mission violate any constitutional principles in `ANTI_GRAVITY.md`?

**Analysis:**
- [ ] Stateless principle
- [ ] Least Privilege principle
- [ ] Contract-Only principle
- [ ] Zero Business Logic principle
- [ ] Fail-Fast principle

**Findings:**
[Detailed analysis of potential violations]

---

### 2.2 Scope Creep Risk
**Question:** Does the mission scope exceed what's necessary?

**Red Flags:**
- Adding features not in requirements
- "While we're at it" additions
- Premature optimizations
- Convenience features

**Findings:**
[Analysis of scope boundaries]

---

### 2.3 Technical Debt Risk
**Question:** Will this mission create maintenance burden?

**Considerations:**
- Code complexity
- Test coverage
- Documentation requirements
- Future refactoring needs

**Findings:**
[Assessment of long-term costs]

---

## III. FAILURE MODE PREDICTION

### 3.1 Most Likely Failure Scenario
**Description:**
[What will probably go wrong]

**Probability:** High | Medium | Low

**Impact:** Critical | Major | Minor

**Mitigation:**
[How to prevent or handle this failure]

---

### 3.2 Worst-Case Failure Scenario
**Description:**
[What's the catastrophic outcome]

**Probability:** High | Medium | Low

**Impact:** Critical | Major | Minor

**Mitigation:**
[Emergency response plan]

---

## IV. TRADE-OFF ANALYSIS

### 4.1 What We Gain
- [Benefit 1]
- [Benefit 2]
- [Benefit 3]

### 4.2 What We Risk
- [Risk 1]
- [Risk 2]
- [Risk 3]

### 4.3 What We Sacrifice
- [Trade-off 1]
- [Trade-off 2]
- [Trade-off 3]

---

## V. ALTERNATIVE APPROACHES

### Alternative 1: [Name]
**Pros:**
- [Advantage 1]
- [Advantage 2]

**Cons:**
- [Disadvantage 1]
- [Disadvantage 2]

**Recommendation:** Prefer | Consider | Reject

---

### Alternative 2: [Name]
**Pros:**
- [Advantage 1]
- [Advantage 2]

**Cons:**
- [Disadvantage 1]
- [Disadvantage 2]

**Recommendation:** Prefer | Consider | Reject

---

## VI. DEPENDENCIES & ASSUMPTIONS

### 6.1 External Dependencies
- [Dependency 1]: Risk level [High/Medium/Low]
- [Dependency 2]: Risk level [High/Medium/Low]

### 6.2 Critical Assumptions
- [Assumption 1]: Validity [Strong/Weak]
- [Assumption 2]: Validity [Strong/Weak]

**What happens if assumptions fail?**
[Contingency analysis]

---

## VII. TESTING REQUIREMENTS

### 7.1 Must-Have Tests
- [ ] [Test scenario 1]
- [ ] [Test scenario 2]
- [ ] [Test scenario 3]

### 7.2 Edge Cases to Cover
- [ ] [Edge case 1]
- [ ] [Edge case 2]

### 7.3 Performance Benchmarks
- [ ] [Benchmark 1]: Target [value]
- [ ] [Benchmark 2]: Target [value]

---

## VIII. ROLLBACK PLAN

**If mission fails, how do we revert?**

**Steps:**
1. [Rollback step 1]
2. [Rollback step 2]
3. [Rollback step 3]

**Data Safety:**
[How to ensure no data loss during rollback]

---

## IX. FINAL RECOMMENDATION

### For Human Architect

**Verdict:** `PASS` | `WARN` | `FAIL`

**Justification:**
[Detailed reasoning for verdict]

**Conditions (if WARN):**
- [Condition 1 that must be met]
- [Condition 2 that must be met]

**Blocking Issues (if FAIL):**
- [Issue 1 that must be resolved]
- [Issue 2 that must be resolved]

---

**Signed:** AGENT S (Skeptic)  
**Authority:** Veto power per IIP v1.1 SOP
