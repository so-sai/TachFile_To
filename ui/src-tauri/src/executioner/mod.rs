/*
 * EXECUTIONER MODULE - Mission 012B Phase 2
 * ========================================
 * Contains all components for mechanical execution:
 * - API: Type definitions (frozen, immutable)
 * - Ledger: SQLite-backed audit trail
 * - Executor: Filesystem operations (future)
 * - Recovery: Startup scan (future)
 */

pub mod api;
pub mod ledger;
pub mod executor;

pub use api::{
    ExecutionWarrant, ExecutionReport, ExecutionError, Executioner, QuiesceSignal,
    FileOrigin, NamingContract, SoftDeleteSpec, PurgeAllProtocol, WarrantState, LedgerEntry,
    current_timestamp, hash_file_id,
};

pub use ledger::{SqliteLedger, LedgerBackend, WarrantEntry, ExecutionEventEntry, ExecutionResult};

pub use executor::FilesystemExecutioner;
