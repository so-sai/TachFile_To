pub mod manager;
pub mod protocol;
pub mod router;

pub use manager::IpcManager;
pub use protocol::{
    BoundingBox, ErrorPayload, ErrorSeverity, ExtractEvidencePayload, HandshakeRequestPayload,
    HandshakeResponsePayload, IpcMessage, MessageType, ParseTablePayload, ProgressPayload,
    SuccessPayload,
};
pub use router::{MessageRouter, RouterResponse};
