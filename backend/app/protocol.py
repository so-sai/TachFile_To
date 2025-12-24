"""
Protocol definitions for Rust-Python IPC
Matches exactly the JSON schema defined in backend_worker_v1.md

This is the SOURCE OF TRUTH for Python side.
Any changes here must be reflected in:
- src-tauri/src/ipc/protocol.rs (Rust)
- src/types/ipc.ts (TypeScript)
"""
from typing import Dict, Any, Optional, List, Tuple
from enum import Enum
from pydantic import BaseModel, Field
import uuid
import os


# ==================== ENUMS ====================

class CommandType(str, Enum):
    """Available commands that Rust can send to Python"""
    EXTRACT_EVIDENCE = "extract_evidence"
    PARSE_TABLES = "parse_tables"
    HEALTH_CHECK = "health_check"
    FORCE_GC = "force_gc"
    SHUTDOWN = "shutdown"


class Priority(str, Enum):
    """Request priority levels"""
    IMMEDIATE = "immediate"   # User clicked, needs <500ms
    NORMAL = "normal"         # User hovered, needs <2s
    BACKGROUND = "background" # Prefetch, can wait


class LifecycleEvent(str, Enum):
    """Worker lifecycle events"""
    READY = "ready"
    OOM_WARNING = "oom_warning"
    SHUTTING_DOWN = "shutting_down"


class ErrorType(str, Enum):
    """Error types matching Rust ErrorType enum"""
    FILE_NOT_FOUND = "file_not_found"
    PAGE_OUT_OF_RANGE = "page_out_of_range"
    MEMORY_EXHAUSTED = "memory_exhausted"
    TIMEOUT_EXCEEDED = "timeout_exceeded"
    PARSING_FAILED = "parsing_failed"
    UNKNOWN = "unknown"


# ==================== REQUEST MODELS ====================

class EvidencePayload(BaseModel):
    """Payload for extract_evidence command"""
    file_path: str
    page_index: int = Field(..., ge=0, description="0-based page index")
    bbox: Tuple[float, float, float, float] = Field(
        ..., 
        description="Bounding box [x, y, width, height] in PDF coords"
    )
    dpi: int = Field(default=150, ge=72, le=300)
    quality: int = Field(default=85, ge=1, le=100, description="JPEG quality")


class WorkerRequest(BaseModel):
    """Request from Rust to Python worker"""
    id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    cmd: CommandType
    payload: Dict[str, Any] = Field(default_factory=dict)
    priority: Priority = Priority.NORMAL
    
    class Config:
        use_enum_values = True


# ==================== RESPONSE MODELS ====================

class PerformanceMetrics(BaseModel):
    """Performance metrics for monitoring"""
    duration_ms: float
    peak_ram_mb: Optional[float] = None


class EvidenceData(BaseModel):
    """Successful evidence extraction result"""
    base64: str
    width: int
    height: int
    mime_type: str = "image/jpeg"


class ErrorDetail(BaseModel):
    """Error details for failed requests"""
    type: ErrorType
    message: str
    traceback: Optional[str] = None


class WorkerResponse(BaseModel):
    """Response from Python worker to Rust"""
    req_id: str
    status: str  # "success" or "error"
    data: Optional[Dict[str, Any]] = None
    error: Optional[ErrorDetail] = None
    perf: Optional[PerformanceMetrics] = None
    
    @classmethod
    def success(
        cls,
        req_id: str,
        data: Dict[str, Any],
        duration_ms: float,
        peak_ram_mb: Optional[float] = None
    ) -> "WorkerResponse":
        return cls(
            req_id=req_id,
            status="success",
            data=data,
            perf=PerformanceMetrics(
                duration_ms=duration_ms,
                peak_ram_mb=peak_ram_mb
            )
        )
    
    @classmethod
    def error(
        cls,
        req_id: str,
        error_type: ErrorType,
        message: str,
        traceback: Optional[str] = None
    ) -> "WorkerResponse":
        return cls(
            req_id=req_id,
            status="error",
            error=ErrorDetail(
                type=error_type,
                message=message,
                traceback=traceback
            )
        )


# ==================== LIFECYCLE MODELS ====================

class ReadyMessage(BaseModel):
    """Initial ready signal from Python to Rust"""
    type: str = "lifecycle"
    event: LifecycleEvent = LifecycleEvent.READY
    version: str
    pid: int = Field(default_factory=os.getpid)
    capabilities: List[str] = Field(default_factory=list)
    worker_id: str = Field(default_factory=lambda: str(uuid.uuid4()))


class OOMWarningMessage(BaseModel):
    """Out of memory warning before worker kills itself"""
    type: str = "lifecycle"
    event: LifecycleEvent = LifecycleEvent.OOM_WARNING
    memory_mb: float
    limit_mb: float
    pid: int = Field(default_factory=os.getpid)


# ==================== HEALTH CHECK ====================

class HealthCheckResponse(BaseModel):
    """Health check response"""
    status: str  # "healthy" or "degraded"
    memory_mb: float
    uptime_seconds: float
    requests_processed: int
    error_count: int
    worker_id: str
