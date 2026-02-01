pub mod janitor;
pub mod gatekeeper;
pub mod docling_bridge;
pub mod audit_log;
pub mod repair_engine;
pub mod diagnostics;
pub mod encoding_normalizer;

pub use janitor::{Janitor, IronJanitor, JanitorReport, JanitorError, CellChange};
pub use gatekeeper::EncodingGatekeeper;
pub use encoding_normalizer::EncodingNormalizer;
pub use iron_table::contract::{EncodingStatus, TableTruth, TableRejection, ProjectTruth, CellValue};
pub use diagnostics::{StructuredRejection, TruthDiff, CellRepair, DiagnosticEngine};
pub use repair_engine::{RepairEngine, LegacyEncodingMode};

