"""
IPC Protocol v1.0 - Python Implementation
Matches specification in IPC-001
"""

from pydantic import BaseModel, Field
from typing import Optional, Any, List, Tuple
from enum import Enum
import time
import uuid


class MessageType(str, Enum):
    CMD_HANDSHAKE = "CMD_HANDSHAKE"
    RES_HANDSHAKE = "RES_HANDSHAKE"
    CMD_PING = "CMD_PING"
    RES_PONG = "RES_PONG"
    CMD_EXTRACT_EVIDENCE = "CMD_EXTRACT_EVIDENCE"
    CMD_PARSE_TABLE = "CMD_PARSE_TABLE"
    RES_SUCCESS = "RES_SUCCESS"
    RES_PROGRESS = "RES_PROGRESS"
    RES_ERROR = "RES_ERROR"
    CMD_SHUTDOWN = "CMD_SHUTDOWN"
    RES_ACK = "RES_ACK"


class CacheHit(str, Enum):
    MEMORY = "memory"
    DISK = "disk"
    MISS = "miss"


class ErrorSeverity(str, Enum):
    FATAL = "fatal"
    ERROR = "error"
    WARNING = "warning"
    INFO = "info"


class IpcMessage(BaseModel):
    """Universal message envelope"""
    protocol_v: str = "1.0.0"
    msg_id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    timestamp: int = Field(default_factory=lambda: int(time.time() * 1000))
    type: MessageType
    payload: Any

    class Config:
        use_enum_values = True


class BoundingBox(BaseModel):
    """PDF bounding box in specified units"""
    x: float
    y: float
    width: float
    height: float
    unit: str = "pt"


class HandshakeRequestPayload(BaseModel):
    rust_version: str
    expected_protocol_v: str
    capabilities_requested: List[str]


class HandshakeResponsePayload(BaseModel):
    worker_pid: int
    docling_version: str
    python_version: str
    capabilities_supported: List[str]
    max_memory_mb: int
    status: str


class ExtractEvidencePayload(BaseModel):
    file_path: str
    page_index: int
    bbox: BoundingBox
    dpi: int = 150
    output_format: Optional[str] = "jpeg"
    quality: Optional[int] = 85


class ParseTablePayload(BaseModel):
    file_path: str
    page_index: int
    hint_bbox: Optional[BoundingBox] = None
    detection_confidence_threshold: float = 0.7
    language: str = "vie"


class SuccessPayload(BaseModel):
    req_id: str
    data: Any
    metadata: Optional[Any] = None


class ErrorPayload(BaseModel):
    req_id: str
    code: str
    severity: ErrorSeverity
    message: str
    details: Optional[Any] = None
    suggested_action: Optional[str] = None
    timestamp: int = Field(default_factory=lambda: int(time.time() * 1000))
    stack_trace: Optional[str] = None


class ProgressPayload(BaseModel):
    req_id: str
    stage: str
    current: int
    total: int
    stage_description: Optional[str] = None
    eta_seconds: Optional[int] = None


class ErrorCodes:
    """Standard error codes for IPC"""
    E_SYS_001 = "E-SYS-001"  # Internal System Error
    E_FILE_001 = "E-FILE-001"  # File Not Found
    E_VAL_001 = "E-VAL-001"  # Validation Error
    E_PROC_001 = "E-PROC-001"  # Processing Error


# Helper functions
def create_message(msg_type: MessageType, payload: Any) -> IpcMessage:
    """Create a new IPC message with auto-generated ID and timestamp"""
    return IpcMessage(type=msg_type, payload=payload)


def create_error_message(
    request_id: str,
    error_code: str,
    message: str,
    severity: ErrorSeverity = ErrorSeverity.ERROR,
    details: Optional[Any] = None,
    suggested_action: Optional[str] = None,
) -> IpcMessage:
    """Create an error response message"""
    payload = ErrorPayload(
        req_id=request_id,
        code=error_code,
        severity=severity,
        message=message,
        details=details,
        suggested_action=suggested_action,
    )
    # Using model_dump (pydantic v2) or dict (pydantic v1) for payload
    payload_data = payload.model_dump() if hasattr(payload, "model_dump") else payload.dict()
    return create_message(MessageType.RES_ERROR, payload_data)


def create_success_message(request_id: str, data: Any, metadata: Optional[Any] = None) -> IpcMessage:
    """Create a success response message"""
    payload = SuccessPayload(req_id=request_id, data=data, metadata=metadata)
    payload_data = payload.model_dump() if hasattr(payload, "model_dump") else payload.dict()
    return create_message(MessageType.RES_SUCCESS, payload_data)


# Add method to IpcMessage for easier serialization
def to_json(self) -> str:
    return self.model_dump_json() if hasattr(self, "model_dump_json") else self.json()

IpcMessage.to_json = to_json
