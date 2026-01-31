# 🛡️ Mission 012A - ResourceCourt

**Status: ✅ COMPLETE (Skeleton Phase)**  
**Tests: 5/5 PASSING**  
**Date: 2026-01-28**

---

## Quick Start

### View Implementation
```bash
cat ui/src-tauri/src/resource_court.rs
```

### Run Tests
```bash
cd ui/src-tauri
cargo test resource_court --lib -- --nocapture
```

**Expected Output:**
```
running 5 tests
test resource_court::tests::test_registry_basic_operations ... ok
test resource_court::tests::test_court_eviction_score_calculation ... ok
test resource_court::tests::test_court_judgment_with_pinned_entry ... ok
test resource_court::tests::test_entropy_calculation ... ok
test resource_court::tests::test_multiple_entries_judgment ... ok

test result: ok. 5 passed
```

---

## Architecture at a Glance

```
CacheRegistry (Observe)
    ↓
ResourceCourt (Judge)
    ↓
Executioner (Execute) ← Future Phase
```

---

## The Four Components You Need to Know

### 1. **CacheEntry**
Metadata about a cached file:
```rust
file_id: "img_123"
file_size_bytes: 1048576
file_count: 1
created_at: 1706395200
last_accessed_at: 1706481600
access_count: 5
user_pinned: false
viewport_distance: 0.5
```

### 2. **EvictionScore** (The Formula)
```
Score = 0.25×Size + 0.25×Age + 0.30×Viewport + 0.20×Entropy
        │              │           │               │
        Size Ratio     Normalized   Distance from   Filesystem
        vs Cache       by 30 days   View (0-1)     Fragmentation
```

### 3. **EvictionVerdict** (The Decision)
```rust
action: RETAIN | MONITOR | SOFT_DELETE | HARD_DELETE
reason: "Size: 0.05, Age: 0.25, Viewport: 0.15, Entropy: 0.00"
score: 0.45
is_reversible: true // false only for HARD_DELETE
```

### 4. **EvictionPolicy** (The Rules)
```rust
max_cache_size_bytes: 500_000_000    // 500 MB
min_age_seconds: 86_400               // 1 day
size_weight: 0.25
age_weight: 0.25
viewport_weight: 0.30
entropy_weight: 0.20
eviction_threshold_critical: 0.8
eviction_threshold_high: 0.6
eviction_threshold_medium: 0.4
```

---

## Usage Example

```rust
use crate::resource_court::*;

// 1. Create registry and court
let mut registry = CacheRegistry::new();
let policy = EvictionPolicy::default();
let mut court = ResourceCourt::new(policy);

// 2. Register files
registry.register_entry(CacheEntry {
    file_id: "cache_001".to_string(),
    file_size_bytes: 100_000_000,  // 100 MB
    file_count: 1,
    created_at: 1706395200,
    last_accessed_at: 1706481600,
    access_count: 5,
    user_pinned: false,
    viewport_distance: 0.5,
});

// 3. Judge all entries
let verdicts = court.judge_entries(&registry, 350_000_000);

// 4. Process verdicts
for verdict in verdicts {
    println!("{}: {} (score: {:.2})", 
        verdict.file_id, 
        format!("{:?}", verdict.action), 
        verdict.score
    );
}
```

---

## Key Properties

### ✅ Deterministic
Same inputs → Same outputs, always.
No randomness, no "adaptive" magic.

### ✅ Transparent
Full score breakdown visible:
- Size component: 0.05
- Age component: 0.25
- Viewport component: 0.15
- Entropy component: 0.00
- **Total: 0.45**

### ✅ User-Centric
Absolute protection for pinned files:
```rust
if entry.user_pinned {
    return RETAIN  // No questions asked
}
```

### ✅ Entropy-Aware
Filesystem doesn't like 50,000 tiny files:
```rust
let entropy = file_count / reference_count;
// High count = high entropy = higher eviction score
```

---

## Severity Levels

| Score | Severity | Action | Example |
|-------|----------|--------|---------|
| < 0.4 | LOW | RETAIN | New file, in viewport |
| 0.4-0.6 | MEDIUM | MONITOR | Old file, far from view |
| 0.6-0.8 | HIGH | SOFT_DELETE | Very old, many small files |
| ≥ 0.8 | CRITICAL | HARD_DELETE | Cache over limit |

---

## Phase 2 Preview

The next phase (012B - Executioner) will add:

1. **Quiesce Protocol** - Stop all readers before deletion
2. **Soft Delete** - Mark for deletion, but recoverable
3. **Hard Delete** - Permanent removal from disk
4. **Transaction Semantics** - Atomic purge-all with rollback
5. **Integration** - Hook into Excel engine

---

## Files

| File | Purpose |
|------|---------|
| `ui/src-tauri/src/resource_court.rs` | Core implementation |
| `docs/specs/MISSION_012A_RESOURCE_COURT.md` | Design spec |
| `docs/MISSION_012A_COMPLETION_REPORT.md` | Full report |

---

## Design Principles

> **"Separate Powers"**  
> Registry observes. Court judges. Executioner acts.

> **"Determinism Over Magic"**  
> Formula-based scoring, not heuristics.

> **"Transparency Is Trust"**  
> Every decision audited and explained.

> **"User First"**  
> Protection is absolute, deletion is careful.

> **"No Auto-Purge"**  
> Irreversible actions require explicit consent.

---

## Questions?

- How is score calculated? → See `calculate_eviction_score()` method
- Why these weights (0.25, 0.25, 0.30, 0.20)? → Desktop use case prioritization
- Can I customize policy? → Yes, pass custom `EvictionPolicy` to `ResourceCourt::new()`
- What about Mission 011 (Parallel)? → Phases are sequential, 012 is prerequisite

---

## Success Criteria Met ✅

- [x] Deterministic scoring formula
- [x] Three-component architecture (Registry, Court, Executioner prep)
- [x] Entropy factor for filesystem health
- [x] User protection mechanism
- [x] Unit tests (5/5 passing)
- [x] Production-ready code
- [x] Comprehensive documentation
- [x] Zero dependencies on filesystem I/O

---

**Status: Ready for Phase 2 Integration** 🚀

By January 28, 2026, Mission 012A has established the constitutional framework for TachFileTo's resource governance. The system is no longer a collection of cached bytes — it's a transparent, auditable system where every eviction decision can be explained and justified.

*"Not just correct, but correct for the right reasons."* 🛡️🦀
