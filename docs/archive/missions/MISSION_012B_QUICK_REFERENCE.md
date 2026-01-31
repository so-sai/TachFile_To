<!-- MISSION 012B QUICK REFERENCE - For Phase 2 Implementation -->

# 🔧 MISSION 012B - QUICK REFERENCE CARD
## Dành Cho Người Thực Hiện Phase 2

**Print this. Tape it to your monitor. Live by it.**

---

## API CONTRACTS (IMMUTABLE - DO NOT MODIFY)

### ExecutionWarrant (No additions!)

```rust
pub struct ExecutionWarrant {
    pub verdict: EvictionVerdict,           // Never modify this
    pub nonce: u64,                         // UNIQUE for this warrant
    pub issued_at: u64,                     // UNIX timestamp (immutable)
    pub signature: String,                  // Court's signature (validate!)
    pub ledger_ref: Option<String>,         // MUST exist in Ledger
}

// FORBIDDEN:
// ❌ pub dry_run: bool
// ❌ pub max_retries: u32
// ❌ pub force_delete: bool
// ❌ pub bypass_protection: bool
```

### Executioner Trait (Single method only!)

```rust
pub trait Executioner {
    fn execute(&mut self, warrant: ExecutionWarrant) 
        -> Result<ExecutionReport, ExecutionError>;
}

// FORBIDDEN:
// ❌ fn validate_warrant()
// ❌ fn dry_run()
// ❌ fn execute_batch()
// ❌ fn can_execute()
```

### QuiesceSignal (Deadline is MANDATORY!)

```rust
pub enum QuiesceSignal {
    None,  // No restrictions
    
    Pending { 
        file_id_hash: u64,        // Hashed file ID (quick compare)
        deadline_unix_sec: u64,   // REQUIRED - UNIX seconds (absolute!)
    },
    
    Global { 
        deadline_unix_sec: u64,   // REQUIRED - UNIX seconds (absolute!)
    },
}

// FORBIDDEN:
// ❌ Pending { file_id_hash, duration_sec }  (relative time!)
// ❌ Pending { file_id_hash }  (no deadline!)
// ❌ Pending { file_id_hash, deadline: Option<u64> }  (optional!)
```

---

## RULES (if broken = system fails)

### ✋ HARD STOPS (Breaking these breaks everything)

| Rule | Why | Cost of Breaking |
|------|-----|------------------|
| **Ledger.append BEFORE execute** | Write-ahead logging | Cross-restart replay attack |
| **Quiesce has deadline (no None)** | Prevent indefinite hang | Workers stuck forever |
| **Executioner doesn't read Policy** | No independent logic | Non-deterministic deletion |
| **Soft-delete = Registry only** | Fail-safe recovery | Can't recover from crash |
| **NamingContract validation** | Ghost vs Alien | Deletes user files |
| **No batching in execute()** | Preserve ordering | Race conditions |

### ⚠️ WARNINGS (Breaking these makes things hard to debug)

| Warning | Impact |
|---------|--------|
| Logging non-deterministically (random UUIDs, timestamps in wrong place) | Audit trail becomes useless |
| Modifying verdict after Court issued it | Violates separation of powers |
| Adding fields to Warrant at runtime | Next person adds more fields |
| Catching all errors silently | Real bugs hidden |

---

## PHASE 2 CHECKLIST (Do these, nothing more)

### Ledger Module

```rust
// Create: src/executioner/ledger.rs

pub struct Ledger {
    // Store these using SQLite or similar
    // ├─ warrant_nonce (PRIMARY KEY)
    // ├─ state (PENDING / EXECUTING / COMMITTED / FAILED)
    // ├─ created_at
    // ├─ completed_at
    // ├─ verdict_json
    // └─ error_detail
}

pub trait LedgerBackend {
    fn append_warrant(&mut self, warrant: &ExecutionWarrant) 
        -> Result<String, Error>;  // Returns ledger_ref
        
    fn update_state(&mut self, nonce: u64, state: WarrantState) 
        -> Result<(), Error>;
        
    fn get_pending_warrants(&self) 
        -> Result<Vec<ExecutionWarrant>, Error>;
        
    fn record_execution(&mut self, report: &ExecutionReport) 
        -> Result<(), Error>;
}
```

**TODO:**
- [ ] Implement LedgerBackend trait
- [ ] Pick storage (SQLite? File-based?)
- [ ] Handle concurrent appends safely
- [ ] Create ledger_ref string format (e.g., "LE_<nonce>_<timestamp>")

---

### Executioner Struct

```rust
// Create: src/executioner/executor.rs

pub struct FilesystemExecutioner {
    ledger: Box<dyn LedgerBackend>,
    log: Logger,
    executed_nonces: HashSet<u64>,  // In-memory cache for runtime dedup
}

impl Executioner for FilesystemExecutioner {
    fn execute(&mut self, warrant: ExecutionWarrant) 
        -> Result<ExecutionReport, ExecutionError> 
    {
        // Step 1: Validate warrant
        if !warrant.is_valid() {
            return Err(ExecutionError::InvalidWarrant);
        }
        
        // Step 2: Check Ledger (no replay!)
        if self.executed_nonces.contains(&warrant.nonce) {
            return Err(ExecutionError::WarrantAlreadyExecuted);
        }
        
        // Step 3: Execute based on action
        let result = match warrant.verdict.action {
            EvictionAction::Retain => {
                // Do nothing
                ExecutionReport { success: true, .. }
            }
            EvictionAction::Monitor => {
                // Log only (no changes)
                ExecutionReport { success: true, .. }
            }
            EvictionAction::SoftDelete => {
                self.soft_delete(&warrant.verdict.file_id)?
            }
            EvictionAction::HardDelete => {
                self.hard_delete(&warrant.verdict.file_id)?
            }
        };
        
        // Step 4: Update ledger
        self.ledger.record_execution(&result)?;
        self.executed_nonces.insert(warrant.nonce);
        
        return Ok(result);
    }
}
```

**TODO:**
- [ ] Implement soft_delete() (Registry + Ledger only)
- [ ] Implement hard_delete() (filesystem ops)
- [ ] Error handling (what if file is locked?)
- [ ] Logging (MUST include warrant nonce in every log line)

---

### Worker Integration (Quiesce Check-in)

```rust
// In: src/commands/render_page.rs (or wherever workers process files)

pub fn render_page_worker(file_id: &str, page_num: u32) -> Result<...> {
    // BEFORE starting any I/O:
    let signal = COURT.get_quiesce_signal(file_id);
    
    match signal {
        QuiesceSignal::None => {
            // Proceed normally
        }
        QuiesceSignal::Pending { deadline_unix_sec, .. } => {
            let now = current_timestamp();
            if now >= deadline_unix_sec {
                // Deadline exceeded!
                COURT.escalate_quiesce(file_id)?;
                return Err(RenderError::QuiesceEscalated);
            }
            // Deadline still valid, yield
            tokio::task::yield_now().await;
            
            // Check again after yield
            let signal = COURT.get_quiesce_signal(file_id);
            if matches!(signal, QuiesceSignal::Pending { .. }) {
                // Still pending, can't proceed
                return Err(RenderError::QuiescePending);
            }
        }
        QuiesceSignal::Global { deadline_unix_sec } => {
            // Global quiesce, must drain
            LOGGER.info("Worker draining due to global quiesce");
            return Err(RenderError::GlobalQuiesce);
        }
    }
    
    // NOW safe to proceed with I/O
    let pdf = open_file(file_id)?;
    let page = pdf.get_page(page_num)?;
    render(page)?;
    
    Ok(...)
}
```

**TODO:**
- [ ] Add Court.get_quiesce_signal() method
- [ ] Add COURT global (Arc<Mutex<ResourceCourt>>)
- [ ] Add Worker.escalate_quiesce() reporting
- [ ] Handle QuiesceSignal::Global drain

---

### Startup Scan Recovery

```rust
// Create: src/executioner/recovery.rs

pub fn startup_scan() -> Result<(), Error> {
    let ledger = Ledger::open()?;
    let registry = CacheRegistry::new();
    
    // Step 1: Check for unfinished warrants
    let pending = ledger.get_pending_warrants()?;
    for warrant in pending {
        // Finish execution
        let mut executor = FilesystemExecutioner::new();
        executor.execute(warrant)?;
    }
    
    // Step 2: Scan disk for Ghost files
    let cache_dir = get_cache_directory()?;
    for entry in fs::read_dir(&cache_dir)? {
        let file_name = entry?.file_name();
        let file_name_str = file_name.to_string_lossy();
        
        // Classify file
        let origin = NamingContract::classify(&file_name_str);
        match origin {
            FileOrigin::Ghost => {
                // Check if in Registry
                if !registry.entries().contains_key(&file_name_str) {
                    // It's a leftover Ghost, delete it
                    fs::remove_file(cache_dir.join(&file_name))?;
                    ledger.log_ghost_cleanup(&file_name_str)?;
                }
            }
            FileOrigin::Alien => {
                // Don't touch!
                LOGGER.warn("Found Alien file: {}", file_name_str);
            }
        }
    }
    
    Ok(())
}
```

**TODO:**
- [ ] Run startup_scan() on app launch
- [ ] Handle cleanup errors gracefully
- [ ] Log Ghost cleanup to Ledger
- [ ] Verify Registry + Ledger consistency

---

## ERROR HANDLING MATRIX

When implementing, handle these:

```
Error: FileNotFound
└─ Executioner: Try to stat() first, log if missing
└─ Recovery: Mark warrant as FAILED, continue

Error: PermissionDenied
└─ Executioner: Log file owner and permissions
└─ Recovery: Add file to "needs-permission-fix" queue (future)

Error: FileLocked (file still being read)
└─ Executioner: Return FileLocked error (do NOT retry)
└─ Recovery: Startup scan will retry, Worker should Yield

Error: Disk Full
└─ Executioner: Return IoError, don't create partial state
└─ Recovery: System administrator must free disk space

Error: Nonce Already Executed
└─ Executioner: Return WarrantAlreadyExecuted (idempotent)
└─ Recovery: This is OK! Ledger deduplication is working
```

---

## LOGGING REQUIREMENTS

Every log line MUST include:

```rust
// ❌ BAD
log::info!("Deleting file");

// ✅ GOOD
log::info!(
    "WARRANT_EXECUTE [nonce={:016x}] action={:?} file_id={} status=start",
    warrant.nonce,
    warrant.verdict.action,
    warrant.verdict.file_id
);

// After execution:
log::info!(
    "WARRANT_EXECUTE [nonce={:016x}] status=complete success={} error={:?}",
    warrant.nonce,
    report.success,
    report.error
);
```

**Why?**
- Forensics: Can grep for nonce and find entire story
- Audit: Regulators can see decision → action chain
- Debugging: Know EXACTLY what happened

---

## TESTING PHASE 2 CODE

These test scenarios MUST pass:

```rust
#[test]
fn test_warrant_idempotence() {
    // Execute same warrant twice → second execution rejected
    let warrant = create_warrant(nonce=42);
    let report1 = executor.execute(warrant.clone()).unwrap();
    let report2 = executor.execute(warrant).unwrap_err();
    assert!(matches!(report2, ExecutionError::WarrantAlreadyExecuted));
}

#[test]
fn test_soft_delete_no_filesystem_changes() {
    // Soft-delete must NOT call fs::remove_file()
    let warrant = create_warrant(action=SoftDelete);
    executor.execute(warrant).unwrap();
    // File must still exist on disk!
    assert!(fs::metadata(file_path).is_ok());
}

#[test]
fn test_hard_delete_requires_ledger_ref() {
    // Warrant without ledger_ref should be rejected
    let mut warrant = create_warrant(action=HardDelete);
    warrant.ledger_ref = None;
    let result = executor.execute(warrant);
    assert!(result.is_err());
}

#[test]
fn test_startup_scan_ghost_cleanup() {
    // File on disk but not in Registry → cleaned up
    let ghost_file = "TFT_abc123_page_001_1234567890.tft_cache";
    create_file_on_disk(ghost_file);
    
    startup_scan().unwrap();
    
    // File must be deleted
    assert!(!fs::metadata(ghost_file).is_ok());
}

#[test]
fn test_startup_scan_alien_protection() {
    // User file on disk → must NOT be touched
    let user_file = "my_important_document.pdf";
    create_file_on_disk(user_file);
    
    startup_scan().unwrap();
    
    // File must still exist!
    assert!(fs::metadata(user_file).is_ok());
}

#[test]
fn test_crash_recovery_at_each_phase() {
    // Simulate crash at P0, P1, P2, ... P6
    // Verify recovery is deterministic at each point
    // (See MISSION_012B_ENFORCEMENT_DESIGN.md for full matrix)
}
```

---

## RED FLAGS (If you see these, STOP and ask)

🚨 **Someone suggests adding `dry_run` parameter**
→ That's a feature addition, not implementation. Reject it.

🚨 **Someone says "Ledger is optional"**
→ No. Ledger is the only thing that makes crash recovery work.

🚨 **Someone says "Let Executioner retry failed deletes"**
→ No. Executioner reports failure, Ledger decides retry policy.

🚨 **Someone wants to batch `execute()` calls**
→ No. Ordering matters. Execute one-at-a-time.

🚨 **Someone wants to skip Naming Contract check**
→ No. That's how Ghost and Alien differ. Non-negotiable.

🚨 **"We'll add deadline validation later"**
→ No. Add it now or it never happens. Workers will hang.

---

## SUCCESS CRITERIA

When Phase 2 is done:

```
✅ All tests pass (both new tests and existing 44 tests)
✅ Code compiles with no warnings
✅ Can trace every file deletion back to a Warrant nonce in Ledger
✅ Crash recovery has been tested at each of 7 points (P0-P6)
✅ No file I/O in soft_delete() function
✅ All error paths include audit logging
✅ Naming Contract is enforced in Ghost cleanup
✅ Workers can be interrupted by Quiesce signal
✅ Zero modifications to API contracts
✅ Code review passes (ask for someone who read MISSION_012B_ENFORCEMENT_DESIGN.md)
```

---

## RESOURCES

- **Design Doc:** [MISSION_012B_ENFORCEMENT_DESIGN.md](../MISSION_012B_ENFORCEMENT_DESIGN.md)
- **API Source:** [executioner.rs](../../ui/src-tauri/src/executioner.rs)
- **Court Source:** [resource_court.rs](../../ui/src-tauri/src/resource_court.rs)
- **This Document:** You're reading it now!

---

## FINAL WORDS

> **"You are not writing a deletion tool. You are implementing the orders of a Judge who cannot be wrong. Your job is to make sure those orders are carried out faithfully and forever recorded."**

Good luck. Don't break the immutable parts. The system will take care of itself.

---

**Last Updated:** 2026-01-28  
**For Phase 2 Implementation:** 2026-01-28 - 2026-02-15 (estimated)  
**Keep this with you at all times.** ✅
