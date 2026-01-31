<!-- Mission 012B - Enforcement Design Complete Specification -->

# 🛡️ MISSION 012B - EXECUTIONER & QUIESCE PROTOCOL
## Bản Giám Định Hoàn Tất: Cơ Chế Hành Pháp Được Khóa

**Build Date:** 2026-01-28  
**Status:** API LOCKED ✅ - Ready for Implementation Phase  
**Principles:** Executioner is "dumb", Quiesce has Deadline, Ghost Files validated  

---

## I. BỘ KHUNG QUY TẮC (ENCAPSULATION BOUNDARIES)

### 1. Three-Power Separation (Tam Quyền Phân Lập)

| Layer | Responsibility | Tools | Constraints |
|-------|----------------|-------|-------------|
| **Court (Tòa)** | Judgment & Verdict | `EvictionScore`, `EvictionVerdict` | Pure Logic, No I/O |
| **Executioner (Đao phủ)** | Execution | `ExecutionWarrant`, `ExecutionReport` | "Vô tri" - No independent decisions |
| **Quiesce (Giao thức)** | Synchronization | `QuiesceSignal`, Deadline | Cooperative yielding, Never blocking |

### 2. Warrant is the ONLY Input to Executioner

```rust
// ✅ ALLOWED
pub fn execute(&mut self, warrant: ExecutionWarrant) -> Result<ExecutionReport, ExecutionError>

// ❌ FORBIDDEN
pub fn execute_with_policy(warrant, policy)  // Direct policy access
pub fn execute_smart(file_id, should_validate)  // Logic beyond warrant
pub fn batch_execute(warrants)  // Batching hides ordering issues
```

---

## II. WRITE-AHEAD WARRANT PROTOCOL (Phòng chống Cross-Restart Replay)

### Problem: How to prevent double-execution after restart?

```
Timeline without WAL:
├─ 08:00:00  Court issues Warrant { nonce=42 }
├─ 08:00:01  Registry.soft_delete("file_123")
├─ 08:00:02  💥 CRASH
├─ 08:00:10  Restart
├─ 08:00:11  Court rebuilds state, issues Warrant { nonce=42 } again
└─ 08:00:12  BUG: Same file deleted twice? Or Executioner refuses second nonce?
```

### Solution: Write-Ahead Ledger

```
Timeline WITH WAL:
├─ 08:00:00  Court issues Warrant { nonce=42 }
├─ 08:00:00  Ledger.append(LedgerEntry { nonce=42, state=PENDING })  ← BEFORE anything else
├─ 08:00:01  Registry.soft_delete("file_123")
├─ 08:00:02  💥 CRASH
├─ 08:00:10  Restart
├─ 08:00:11  Ledger.scan() finds { nonce=42, state=PENDING }
├─ 08:00:11  Court reconstructs Warrant from Ledger
└─ 08:00:12  Executioner executes ONCE (state already PENDING, not new)
```

### Implementation Contract

**IMMUTABLE RULE:**
```
Ledger.append(warrant_entry) MUST happen BEFORE any other action
├─ Registry modification
├─ File I/O
└─ Quiesce signal
```

**Nonce Deduplication:**
```
Executioner checks: Is nonce in Ledger?
├─ No  → Create new entry, execute
├─ Yes, state=PENDING  → Execute again (safe idempotent)
└─ Yes, state=COMMITTED  → Reject (already done)
```

---

## III. SOFT-DELETE DEFINITION (Trục Xuất Pháp Lý)

### NOT File System Deletion

```
Soft-Delete in TachFileTo:
┌─────────────────────────────────────────┐
│ Action 1: Registry.remove(file_id)      │  ← Logical exile (memory only)
├─────────────────────────────────────────┤
│ Action 2: Ledger.mark_ghost(file_id)   │  ← Record in audit trail
├─────────────────────────────────────────┤
│ ❌ NOT: fs::remove_file() / rename()    │  ← Physical deletion NOT allowed
└─────────────────────────────────────────┘
```

### Why?

**Scenario:** System crashes 0.5 seconds after soft-delete

```
With soft-delete (Registry removed):
├─ Registry is clean
├─ File still exists on disk (Ghost)
└─ Startup Scan finds it → cleans up → OK

With hard-delete (file removed):
├─ Registry claims file is deleted
├─ File is partially deleted (0.5 sec into multi-second I/O)
├─ Ledger shows deletion is incomplete
└─ Startup: confused state (Registry and Filesystem mismatch)
```

### Soft-Delete Contract

```rust
pub fn soft_delete_entry(file_id: &str) -> Result<(), Error> {
    // Step 1: Log intention to Ledger
    ledger.mark_pending_exile(file_id)?;
    
    // Step 2: Remove from Registry (not queryable anymore)
    registry.remove(file_id)?;
    
    // Step 3: Ledger updates to "exile complete" (no file lookup needed)
    ledger.mark_exile_committed(file_id)?;
    
    return Ok(());
}
```

---

## IV. PROOF OF ORIGIN - GHOST VS. ALIEN (Phân biệt Ghost Files)

### The Risk

```
User's cache directory: /home/user/.cache/app/
Contains:
├─ TFT_abc123_page001_1672531200.tft_cache      ← TachFileTo file (Ghost)
├─ backup_important_doc.pdf                     ← User's file (Alien)
└─ system_temp_12345.tmp                        ← Unknown origin

If Startup Scan deletes "anything not in Registry":
├─ Ghost file deleted → OK
├─ Alien file deleted → ❌ USER'S DATA LOSS
```

### Naming Contract (Immutable)

**All valid TachFileTo cache files MUST match:**
```
TFT_<ContentHash>_<PageID>_<CreatedTimestamp>.tft_cache

Example:
TFT_a1b2c3d4e5f6g7h8_page_001_1672531200.tft_cache
└── Prefix: TFT_
└── Hash: a1b2c3d4e5f6g7h8 (SHA256 of content)
└── Page: page_001
└── Timestamp: 1672531200 (UNIX seconds)
└── Suffix: .tft_cache
```

### Validation Logic

```rust
pub fn classify_file(name: &str) -> FileOrigin {
    if name.starts_with("TFT_") && name.ends_with(".tft_cache") {
        if parse_timestamp(name).is_ok() {
            return FileOrigin::Ghost;  // Safe to delete if not in Registry
        }
    }
    return FileOrigin::Alien;  // HANDS OFF
}
```

### Startup Scan Behavior

```
For each file in cache directory:
├─ classify(file_name)
├─ If Ghost:
│  └─ Not in Registry? → Hard-delete (it's a leftover from previous crash)
├─ If Alien:
│  └─ Log warning, DO NOT TOUCH
└─ Update progress in Ledger
```

---

## V. QUIESCE DEADLINE - PREVENTING INDEFINITE SUSPENSION

### Problem: Quiesce without Deadline

```
Timeline:
08:00:00  Court.quiesce(file_id="doc.pdf")  // No deadline
08:00:01  Worker 1: sees Quiesce, yields
08:00:02  Worker 2: sees Quiesce, yields
08:00:03  ... (all workers yielding indefinitely)
08:01:00  Court forgets to revoke quiesce
08:02:00  🔒 System is LOCKED (not deadlocked, just rude)
```

### Solution: Hard Deadline

```rust
QuiesceSignal::Pending {
    file_id_hash: hash("doc.pdf"),
    deadline_unix_sec: 1672531260,  // Must complete by this time
}
```

### Worker Contract

```rust
pub fn check_quiesce(file_id: &str) -> Action {
    signal = court.get_quiesce_signal(file_id);
    
    match signal {
        None => return Action::Proceed,
        Pending { deadline, .. } => {
            if now() < deadline {
                return Action::Yield;  // Cooperative, not blocking
            } else {
                return Action::Escalate;  // Deadline exceeded, escalate!
            }
        }
        Global { deadline } => {
            if now() < deadline {
                return Action::Drain;  // Finish current task, exit
            } else {
                return Action::ForceExit;  // Timeout, exit NOW
            }
        }
    }
}
```

### Escalation Protocol

```
If Worker reports deadline exceeded:
├─ Court.escalate(file_id, reason="timeout")
├─ Court options:
│  ├─ 1. Extend deadline (if still waiting for I/O)
│  ├─ 2. Force kill worker (if deadlocked)
│  └─ 3. Revert quiesce (if error on Court's side)
└─ Log escalation to Ledger
```

---

## VI. PROTOCOL 000 - PURGE-ALL (Giao Thức Hủy Diệt Hàng Loạt)

### NOT a simple `for` loop!

Must follow **2-Phase Commit + 4-Phase Execution**:

```
Phase 0: Pre-check
├─ Verify purge_all is enabled in policy
└─ No concurrent purges allowed

Phase 1: Quiesce (System Freeze)
├─ Court issues: QuiesceSignal::Global { deadline: now + 30s }
├─ Workers drain (finish current tasks, don't start new ones)
├─ Wait for all workers to report "drained"
└─ Ledger records: "System in Global Quiesce"

Phase 2: Collect Targets
├─ Registry.entries() → all file IDs to delete
├─ For each: create ExecutionWarrant with nonce
├─ Batch insert into Ledger with state=PENDING
└─ Ledger records: "Targets locked for deletion"

Phase 3: Clear Registry (Logical)
├─ Registry.clear()
├─ Ledger records: "Registry cleared" (irreversible point)
├─ If crash here: startup sees empty Registry, Ledger has targets
│  └─ Startup knows: "delete Ghost files matching these targets"

Phase 4: Execute Deletion
├─ For each warrant in Ledger (state=PENDING):
│  ├─ Executioner.execute(warrant)
│  ├─ Warrant state → COMMITTED (if success)
│  └─ Warrant state → FAILED (if error, log and continue)
└─ Ledger records all results

Phase 5: Cleanup
├─ Revoke Global Quiesce
├─ Workers resume
└─ Ledger records: "Purge-All completed"
```

### Failure Modes

| Crash Point | Registry State | Ledger State | Recovery |
|-------------|---|---|---|
| P1 (Quiesce) | Intact | Quiesce pending | Restart: revoke or retry quiesce |
| P2 (Collect) | Intact | Targets pending | Restart: collect again, overwrite ledger |
| P3 (Clear) | **Empty** | Targets pending | Restart: scan disk, delete Ghost files matching targets |
| P4 (Execute) | Empty | Some COMMITTED, some PENDING | Restart: finish executing PENDING warrants |
| P5 (Cleanup) | Empty | All COMMITTED | Restart: revoke quiesce, resume workers |

---

## VII. FAILURE SIMULATION MATRIX (Ma Trận Thảm Họa)

**Goal:** Prove that at NO point can Registry and Ledger become **logically inconsistent**.

### Definitions
- **Registry Consistency:** Entry exists in Registry ↔ Entry NOT marked as deleted in Ledger
- **Ledger Consistency:** Warrant state matches execution reality
- **System Consistency:** Registry + Ledger + Filesystem agree on "truth"

### Matrix: All Crash Points

```
Crash Point Matrix (7 critical points):

P0: Court issues warrant
└─ Registry: unchanged | Ledger: empty | Verdict: none
   Recovery: Court rebuilds and re-issues → Idempotent ✅

P1: Ledger.append(warrant) COMPLETE
└─ Registry: unchanged | Ledger: PENDING | Verdict: issued but not acted upon
   Recovery: Executioner reads Ledger, executes PENDING warrants ✅

P2: Registry.soft_delete(file_id) COMPLETE
└─ Registry: DELETED | Ledger: PENDING → execution | Verdict: no longer queryable
   Recovery: Startup marks file as Ghost, cleans up ✅

P3: Ledger.mark_committed(warrant) COMPLETE
└─ Registry: DELETED | Ledger: COMMITTED | Verdict: deletion is final
   Recovery: File is Ghost, Startup cleans up ✅

P4: Hard-delete begins (file still on disk)
└─ Registry: DELETED | Ledger: EXECUTING | Verdict: removal in progress
   Recovery: File partially deleted, Startup scan finds incomplete, deletes remainder ✅

P5: Hard-delete COMPLETE (file removed from disk)
└─ Registry: DELETED | Ledger: COMMITTED | Verdict: completely gone
   Recovery: File is gone, Registry knows it's gone → Consistent ✅

P6: Ledger.mark_final_cleanup() COMPLETE
└─ Registry: DELETED | Ledger: ARCHIVED | Verdict: deletion is audited
   Recovery: Warrant history preserved, no ambiguity ✅
```

### Consistency Proof

**Invariant:** At every crash point:
```
If file_id is in Registry:
├─ Ledger does NOT show it as deleted
└─ Filesystem has file available

If file_id is NOT in Registry:
├─ Ledger shows it as pending/committed deletion
└─ Filesystem may have Ghost file (will be cleaned up)

If file_id marked COMMITTED in Ledger:
├─ Must NOT be in Registry
├─ File must be removed from Filesystem eventually
└─ Or marked as awaiting cleanup
```

**Consequence:** Startup recovery is **always deterministic and safe**.

---

## VIII. BOUNDARY VIOLATIONS (What WILL Break This System)

### 🚫 Red Lines (Absolute No-Go)

1. **Executioner reads `EvictionPolicy` directly**
   - Leads to independent judgment
   - Breaks auditability

2. **Quiesce signal without deadline**
   - Can lead to indefinite suspension
   - Violates responsiveness contract

3. **Soft-delete includes physical file operations**
   - `fs::remove_file()`, `fs::rename()` inside soft-delete
   - Breaks failure recovery guarantees

4. **Nonce reuse across restarts**
   - Without Ledger prefix check
   - Allows double-execution

5. **Purge-All without 2-Phase Commit**
   - Registry cleared before Ledger confirmation
   - Allows inconsistent state

6. **Ghost file cleanup without Naming Contract**
   - Deletes Alien files
   - Data loss for user

7. **Worker doesn't check Quiesce before I/O**
   - Can execute on files marked for deletion
   - Race conditions

8. **Executioner returns modified verdict**
   - Changes action from `HardDelete` to `SoftDelete`
   - Court's judgment is overridden

---

## IX. API SUMMARY (Khóa Chặt)

### ExecutionWarrant
```rust
pub struct ExecutionWarrant {
    pub verdict: EvictionVerdict,           // Immutable (from Court)
    pub nonce: u64,                         // Unique, prevents replay
    pub issued_at: u64,                     // Proof of issuance time
    pub signature: String,                  // Future: HMAC by Court
    pub ledger_ref: Option<String>,         // Must exist in Ledger
}
```

### Executioner Trait (SINGLE METHOD)
```rust
pub trait Executioner {
    fn execute(&mut self, warrant: ExecutionWarrant) 
        -> Result<ExecutionReport, ExecutionError>;
}
```

### QuiesceSignal
```rust
pub enum QuiesceSignal {
    None,
    Pending { 
        file_id_hash: u64, 
        deadline_unix_sec: u64,  // REQUIRED
    },
    Global { 
        deadline_unix_sec: u64,  // REQUIRED
    },
}
```

### NamingContract (Proof of Origin)
```rust
pub struct NamingContract;
impl NamingContract {
    pub fn validate(file_name: &str) -> (bool, Vec<String>)
    pub fn classify(file_name: &str) -> FileOrigin
}
```

---

## X. IMPLEMENTATION CHECKLIST (Next Phase)

When implementing Mission 012B execution mechanics:

- [ ] **Ledger Module:**
  - [ ] `append(warrant_entry)` is atomic
  - [ ] Nonce deduplication on read
  - [ ] State transitions: PENDING → EXECUTING → COMMITTED/FAILED
  
- [ ] **Executioner Struct (not trait impl yet):**
  - [ ] Validate warrant signature
  - [ ] Check Ledger for nonce existence
  - [ ] Refuse duplicate nonce with COMMITTED state
  - [ ] Log every attempt to Ledger
  
- [ ] **Soft-Delete Logic:**
  - [ ] Registry removal first
  - [ ] No physical file operations
  - [ ] Ledger update
  
- [ ] **Quiesce Worker Integration:**
  - [ ] Worker checks signal before every file I/O
  - [ ] Respect deadline (no infinite yield)
  - [ ] Report when yielded to Ledger
  
- [ ] **Ghost File Cleanup:**
  - [ ] Scan with Naming Contract validation
  - [ ] Delete only files matching TFT_ pattern
  - [ ] Log each deletion
  
- [ ] **Tests for Failure Modes:**
  - [ ] Simulate crash at each P0-P6 point
  - [ ] Verify recovery is deterministic
  - [ ] Check Registry-Ledger consistency

---

## XI. DECLARATION (Tuyên Bố)

> **"Hệ thống này đã vượt qua ngưỡng của 'Tool' để trở thành một 'Định chế Kỹ thuật'."**
>
> **"The system is now audit-grade. Every decision can be traced. Every failure can be recovered from. And no user data will be lost due to a bug we can't explain."**

**API Status:** 🔒 **LOCKED FOR PRODUCTION**

No further changes to ExecutionWarrant, Executioner trait, or QuiesceSignal are permitted without a formal Architectural Review Board decision.

---

**Compiled by:** Architectural Council  
**Date:** 2026-01-28  
**Mission:** 012B Phase 1 (API Design Complete)
