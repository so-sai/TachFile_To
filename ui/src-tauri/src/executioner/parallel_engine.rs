/*
 * PARALLEL ENGINE - MISSION 011
 * ==============================
 * 
 * "The Turbo" - High-throughput ingestion governed by Discipline.
 * 
 * Features:
 * 1. ParallelDispatcher: Work-stealing task distributor.
 * 2. Disciplined Workers: Parallel threads respecting Quiesce & Frame budgets.
 * 3. Atomic Registry: DashMap-backed metadata tracking.
 * 4. Concurrent Ledger: Individual SQLite connections with WAL.
 */

use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::path::PathBuf;
use crossbeam_channel::{unbounded, Sender, Receiver};

use crate::resource_court::{CacheRegistry, CacheEntry};
use crate::executioner::ledger::{SqliteLedger, LedgerBackend};
use crate::executioner::discipline::{DisciplineGuard, DisciplineDecision, FrameBudget, PressureMonitor};
use crate::executioner::api::{QuiesceSignal, current_timestamp};

// ============================================================
// 1. WORK UNIT
// ============================================================

#[derive(Debug, Clone)]
pub enum WorkUnit {
    IngestFile {
        file_id: String,
        path: PathBuf,
        size: u64,
    },
    RenderPage {
        file_id: String,
        page_index: usize,
    },
    // Future: Add more units
}

// ============================================================
// 2. PARALLEL WORKER
// ============================================================

pub struct ParallelWorker {
    id: String,
    ledger_path: PathBuf,
    registry: Arc<CacheRegistry>,
}

impl ParallelWorker {
    pub fn spawn(
        id: String,
        ledger_path: PathBuf,
        registry: Arc<CacheRegistry>,
        task_rx: Receiver<WorkUnit>,
        signal: Arc<QuiesceSignal>, // Shared signal from Court
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            let mut worker = ParallelWorker { id, ledger_path, registry };
            worker.run(task_rx, signal);
        })
    }

    fn run(&mut self, rx: Receiver<WorkUnit>, signal: Arc<QuiesceSignal>) {
        // Init individual ledger connection
        let mut ledger = match SqliteLedger::open(&self.ledger_path) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Worker {} failed to open ledger: {:?}", self.id, e);
                return;
            }
        };

        let mut pressure_monitor = PressureMonitor::new();

        while let Ok(unit) = rx.recv() {
            // 1. Setup Discipline Guard for this Work Unit
            let budget = FrameBudget::default_60fps();
            let file_id_hash = match &unit {
                WorkUnit::IngestFile { file_id, .. } => crate::executioner::api::hash_file_id(file_id),
                WorkUnit::RenderPage { file_id, .. } => crate::executioner::api::hash_file_id(file_id),
            };

            let mut guard = DisciplineGuard {
                signal: &signal,
                file_id_hash,
                budget: &budget,
                pressure: &mut pressure_monitor,
            };

            // 2. Check Discipline BEFORE work
            match guard.checkpoint() {
                DisciplineDecision::Abort => break,
                DisciplineDecision::Yield => thread::yield_now(),
                _ => {}
            }

            // 3. Execute Work
            self.execute_unit(unit, &mut ledger, &mut guard);
            
            // 4. Check Discipline AFTER work
            if let DisciplineDecision::Abort = guard.checkpoint() {
                break;
            }
        }
    }

    fn execute_unit<L: LedgerBackend>(&self, unit: WorkUnit, _ledger: &mut L, _guard: &mut DisciplineGuard) {
        match unit {
            WorkUnit::IngestFile { file_id, path, size } => {
                // Register in Registry
                self.registry.register_entry(CacheEntry {
                    file_id: file_id.clone(),
                    file_path: path.to_string_lossy().to_string(),
                    file_size_bytes: size,
                    file_count: 1,
                    created_at: current_timestamp(),
                    last_accessed_at: current_timestamp(),
                    access_count: 0,
                    user_pinned: false,
                    viewport_distance: 1.0, 
                });

                // Future: Record in Ledger if it was an "Execution" (like a transfer)
                // For now, simple registration.
            }
            WorkUnit::RenderPage { file_id, page_index } => {
                // Placeholder for real rendering
                thread::sleep(std::time::Duration::from_millis(2)); // Simulated work
                println!("Worker {} rendered {} page {}", self.id, file_id, page_index);
            }
        }
    }
}

// ============================================================
// 3. PARALLEL DISPATCHER
// ============================================================

pub struct ParallelDispatcher {
    task_tx: Sender<WorkUnit>,
    worker_handles: Vec<JoinHandle<()>>,
}

impl ParallelDispatcher {
    pub fn new(
        ledger_path: PathBuf,
        registry: Arc<CacheRegistry>,
        signal: Arc<QuiesceSignal>,
        num_workers: usize,
    ) -> Self {
        let (tx, rx) = unbounded();
        let mut handles = Vec::new();

        for i in 0..num_workers {
            let handle = ParallelWorker::spawn(
                format!("WORKER_{}", i),
                ledger_path.clone(),
                registry.clone(),
                rx.clone(),
                signal.clone(),
            );
            handles.push(handle);
        }

        Self {
            task_tx: tx,
            worker_handles: handles,
        }
    }

    pub fn dispatch(&self, unit: WorkUnit) {
        let _ = self.task_tx.send(unit);
    }

    pub fn shutdown(self) {
        drop(self.task_tx); // Closes the channel
        for handle in self.worker_handles {
            let _ = handle.join();
        }
    }
}

// ============================================================
// 4. TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_dispatcher_basic_throughput() {
        let tmp = tempdir().unwrap();
        let ledger_path = tmp.path().join("ledger.db");
        let registry = Arc::new(CacheRegistry::new());
        let signal = Arc::new(QuiesceSignal::None);
        
        let dispatcher = ParallelDispatcher::new(ledger_path, registry.clone(), signal, 4);
        
        // Dispatch 100 units
        for i in 0..100 {
            dispatcher.dispatch(WorkUnit::IngestFile {
                file_id: format!("file_{}", i),
                path: PathBuf::from(format!("path_{}", i)),
                size: 1024,
            });
        }
        
        dispatcher.shutdown();
        
        assert_eq!(registry.stats().entry_count, 100);
    }

    #[test]
    fn test_parallel_ledger_contention_stress() {
        let tmp = tempdir().unwrap();
        let ledger_path = tmp.path().join("ledger.db");
        
        // Init schema first (sequential)
        { SqliteLedger::open(&ledger_path).unwrap(); }

        let registry = Arc::new(CacheRegistry::new());
        let signal = Arc::new(QuiesceSignal::None);
        let dispatcher = ParallelDispatcher::new(ledger_path.clone(), registry, signal, 8);

        // Workers don't record ingestion in ledger yet in basic execute_unit,
        // let's manually stress ledger from multiple threads using a helper.
        
        let mut handles = Vec::new();
        for i in 0..10 {
            let lp = ledger_path.clone();
            handles.push(thread::spawn(move || {
                let mut led = SqliteLedger::open(lp).unwrap();
                for j in 0..50 {
                    let warrant = WarrantEntry {
                        nonce: format!("W_{}_{}", i, j),
                        issued_at_unix: 1000,
                        target_path: format!("TFT_file_{}_{}.tft_cache", i, j),
                        action: "SOFT_DELETE".to_string(),
                        signature: vec![0],
                        court_version: "1.0".to_string(),
                    };
                    let _ = led.append_warrant(&warrant);
                }
            }));
        }

        for h in handles { h.join().unwrap(); }
        
        // Verify ledger integrity and count
        let mut led = SqliteLedger::open(ledger_path).unwrap();
        let pending = led.get_pending_warrants().unwrap();
        assert_eq!(pending.len(), 500);
        led.verify_integrity().unwrap();
        
        dispatcher.shutdown();
    }
}
