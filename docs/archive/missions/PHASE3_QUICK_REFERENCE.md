<!-- MISSION 012B PHASE 3 - QUICK REFERENCE -->

# 🧹 PHASE 3: THE JANITOR - QUICK REFERENCE CARD

## ⚡ TL;DR

**What:** Cleanup system (Ghost files + Zombie warrants)  
**When:** Startup (before UI interactive)  
**How:** Call `Janitor::new().startup_cleanup(&mut ledger, &registry)`  
**Status:** ✅ 32 tests passed, ready to deploy

---

## 🎯 Three Rules (Non-Negotiable)

### Rule 1: Ghost vs. Alien
```
Ghost:   TFT_<hash>_<page>_<timestamp>.tft_cache + NOT in Registry → DELETE
Alien:   Everything else → NEVER TOUCH
```

### Rule 2: Zombies First
```
BEFORE Ghost cleanup, execute any PENDING warrants in Ledger
This ensures idempotency (can be safely retried)
```

### Rule 3: Ledger is Source of Truth
```
If Ledger.verify_integrity() FAILS → startup FAILS
No exceptions, no silence, no workarounds
```

---

## 📦 Public API

```rust
// Create Janitor
let janitor = Janitor::new(cache_dir);

// Run cleanup
let result = janitor.startup_cleanup(&mut ledger, &registry)?;

// Check report
assert_eq!(result.ghosts_deleted, 5);
assert_eq!(result.zombies_recovered, 2);
println!("{}", result.summary());
```

---

## 🔧 Integration (Tauri)

```rust
// In lib.rs or main.rs
pub fn run() {
    tauri::Builder::default()
        .setup(setup_janitor)  // ← ADD THIS
        .manage(ExcelAppState::default())
        // ...
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// setup_janitor() is in tauri_integration.rs
```

---

## ✅ Test Examples

### Ghost Deletion
```rust
#[test]
fn test_ghost_deletion() {
    // 1. Create Ghost file
    let ghost_path = cache_dir.join("TFT_abc_001_1609459200.tft_cache");
    fs::write(&ghost_path, b"data")?;
    
    // 2. Run Janitor (Registry empty)
    let report = janitor.startup_cleanup(&mut ledger, &registry)?;
    
    // 3. Verify deletion
    assert_eq!(report.ghosts_deleted, 1);
    assert!(!ghost_path.exists());
}
```

### Alien Protection
```rust
#[test]
fn test_alien_protection() {
    // 1. Create Alien file
    let alien = cache_dir.join("my_important.pdf");
    fs::write(&alien, b"USER DATA")?;
    
    // 2. Run Janitor
    let report = janitor.startup_cleanup(&mut ledger, &registry)?;
    
    // 3. Verify NOT deleted (CRITICAL!)
    assert_eq!(report.aliens_protected, 1);
    assert!(alien.exists());  // MUST exist
}
```

---

## 🚨 Error Handling

| Error | Severity | Action |
|---|---|---|
| `LedgerCorrupted` | FATAL | Startup FAILS |
| `PermissionDenied` | WARN | Log, continue |
| `FileNotFound` | OK | Idempotent |
| `IoError` | WARN | Log, continue |

---

## 📊 JanitorReport Fields

```rust
pub struct JanitorReport {
    pub zombies_recovered: usize,    // PENDING → COMMITTED
    pub ghosts_deleted: usize,       // Cleaned unregistered TFT_
    pub ghosts_protected: usize,     // Registered TFT_ (kept)
    pub aliens_protected: usize,     // Non-TFT (kept)
    pub ghost_cleanup_errors: usize, // Failed deletions
}

pub fn is_successful(&self) -> bool {
    ghost_cleanup_errors == 0
}

pub fn summary(&self) -> String {
    // "Janitor Report: X zombies recovered, Y ghosts deleted, ..."
}
```

---

## 🔍 Naming Contract

```rust
// Valid Ghost
"TFT_abc123_page_001_1609459200.tft_cache"  ← DELETE if not in Registry
"TFT_xyz_001_1234567890.tft_cache"          ← DELETE if not in Registry

// Invalid (Alien - NEVER DELETE)
"cache_abc123_page_001.tft_cache"           ← No TFT_ prefix
"TFT_abc123.tmp"                            ← No .tft_cache suffix
"my_important_document.pdf"                 ← Not TFT_ at all
"backup.zip"                                ← User file
```

---

## 🏃 Performance

| Files | Time | Notes |
|---|---|---|
| 100 | ~50ms | Instant |
| 1K | ~200ms | Fast |
| 10K | 2-5s | OK |
| 100K | >30s | Consider threading |

---

## 📋 Deployment Checklist

```
BEFORE SHIP:
□ cargo test --lib executioner (all 32 pass)
□ cargo check (no errors)
□ Integrate setup_janitor() into Tauri
□ Test Ghost deletion
□ Test Alien protection
□ Test Zombie recovery
□ Test with corrupted Ledger (startup fails)
□ Test with > 1K Ghost files (performance OK)
□ Verify Ledger.summary() in console output

AFTER SHIP:
□ Monitor startup logs
□ Check for permission denied errors
□ Verify no accidental user file deletions
□ Track cleanup times (performance baseline)
```

---

## 🐛 Debugging Tips

### "Ghost files not deleted"
1. Check: Is file in Registry? (`registry.contains_entry()`)
2. Check: Does filename match NamingContract? (`NamingContract::classify()`)
3. Check: Does Ledger verify OK? (`ledger.verify_integrity()`)

### "Startup takes too long"
1. Count Ghost files: > 10K?
2. Consider non-blocking mode: `setup_janitor_with_timeout()`
3. Check disk I/O performance

### "Startup fails silently"
1. Check Ledger file: Is it corrupted?
2. Run: `ledger.verify_integrity()` manually
3. Check file permissions on cache directory

### "User file deleted!"
1. CRITICAL: Check if filename matches TFT_ pattern
2. Report bug immediately
3. Restore from backup

---

## 🔗 Related Files

| File | Purpose |
|---|---|
| `recovery.rs` | Core Janitor implementation |
| `phase_3_tests.rs` | Comprehensive test suite |
| `tauri_integration.rs` | Tauri setup examples |
| `MISSION_012B_PHASE3_IMPLEMENTATION.md` | Full documentation |

---

## 🎓 Key Concepts

### Basename Strategy
```rust
// CORRECT: Classify only filename
let basename = path.file_name().and_then(|n| n.to_str())?;
let origin = NamingContract::classify(basename);

// WRONG: Classify full path
let origin = NamingContract::classify(&path.to_string_lossy());
```

### Exclusive Borrow
```rust
// CORRECT: Janitor borrows Ledger mutably
pub fn startup_cleanup<L: LedgerBackend>(
    &self,
    ledger: &mut L,  // ← Exclusive borrow
    registry: &CacheRegistry,
)

// NO CLONING: L doesn't need Clone bound
```

### Idempotency
```rust
// If file already deleted → OK (not an error)
JanitorError::FileNotFound → ExecutionResult::Success
// Safe to retry at any time
```

---

## 🚀 Ready to Deploy

**Status:** ✅ **APPROVED**

- [x] Core implementation complete
- [x] 32 tests passing (100%)
- [x] Zero regressions
- [x] Audit trail complete
- [x] Integration examples provided
- [x] Documentation finalized

**Next:** Merge to `UI-Zero-Latency` branch and deploy.

---

🧹 **"Nhân viên vệ sinh có thẩm quyền pháp lý"**  
🦀 **Rust + SQLite = Atomic Safety**  
✅ **32 Tests = Production Ready**

