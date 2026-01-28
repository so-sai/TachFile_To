/*
 * EXECUTIONER MODULE - Mission 012B Phase 2 & Phase 3
 * ========================================
 * Contains all components for mechanical execution:
 * - API: Type definitions (frozen, immutable)
 * - Ledger: SQLite-backed audit trail
 * - Executor: Filesystem operations (Phase 2)
 * - Recovery: Startup scan & Janitor (Phase 3)
 */

pub mod api;
pub mod ledger;
pub mod executor;
pub mod recovery;
pub mod discipline;
pub mod parallel_engine;

#[cfg(test)]
mod phase_3_tests;

pub use api::{
    ExecutionWarrant, ExecutionReport, ExecutionError, Executioner, QuiesceSignal,
    FileOrigin, NamingContract, SoftDeleteSpec, PurgeAllProtocol, WarrantState, LedgerEntry,
    current_timestamp, hash_file_id,
};

pub use ledger::{SqliteLedger, LedgerBackend, WarrantEntry, ExecutionEventEntry, ExecutionResult};

pub use executor::FilesystemExecutioner;

pub use recovery::{Janitor, JanitorReport, JanitorError};
pub use discipline::{
    QuiesceGate, QuiesceDecision, FrameBudget, BudgetDecision,
    PressureMonitor, PressureDecision, DisciplineGuard, DisciplineDecision,
};
