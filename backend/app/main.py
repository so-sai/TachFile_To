"""
TachFileTo Python Worker - Main Entry Point
IPC Protocol v1.0 Compliant
"""

import sys
import json
import logging
import traceback
import os
import time
from typing import Any

# Import protocol definitions
from app.protocol import (
    IpcMessage, MessageType, 
    ExtractEvidencePayload, ParseTablePayload,
    create_message, create_error_message, create_success_message,
    ErrorCodes, CacheHit
)
from app.engine.extractor import EvidenceExtractor  # Defines mock if needed, but we override

# Configure Logging: CRITICAL - Level INFO, Stream STDERR
logging.basicConfig(
    stream=sys.stderr,
    level=logging.INFO,
    format='[PY-WORKER] %(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger("worker")


def main():
    """Main worker loop - reads from stdin, writes to stdout"""
    logger.info("=" * 60)
    logger.info("🔥 Python Worker Process Started")
    logger.info(f"   PID: {os.getpid()}")
    logger.info(f"   Protocol Version: 1.0.0")
    logger.info("=" * 60)

    # 1. Initialize Engine (Real Implementation)
    try:
        from app.engine.evidence_extractor import RealEvidenceExtractor
        from app.engine.cache_manager import EvidenceCache
        
        extractor = RealEvidenceExtractor()
        cache_manager = EvidenceCache()  # Defaults to data/evidence_cache.db
        logger.info("✅ Engine & Cache initialized (Real + PyMuPDF + SQLite)")
    except Exception as e:
        logger.critical(f"❌ Failed to initialize Engine: {e}")
        logger.critical(traceback.format_exc())
        sys.exit(1)

    # 2. Main Event Loop
    logger.info("📡 Listening for commands on stdin...")
    
    while True:
        try:
            # Read one line from stdin (blocking)
            line = sys.stdin.readline()
            
            # Check for EOF (stdin closed by parent process)
            if not line:
                logger.info("📪 Stdin closed (EOF). Worker exiting gracefully.")
                extractor.close()
                break
            
            # Parse JSON envelope
            try:
                raw_msg = json.loads(line)
                msg = IpcMessage(**raw_msg)
            except json.JSONDecodeError as e:
                logger.error(f"❌ Invalid JSON received: {e}")
                continue
            except Exception as e:
                logger.error(f"❌ Protocol validation error: {e}")
                continue

            logger.info(f"📨 Received: {msg.type} (msg_id: {msg.msg_id})")

            # 3. Route message to appropriate handler
            response = None
            
            try:
                if msg.type == MessageType.CMD_HANDSHAKE:
                    response = handle_handshake(msg)
                
                elif msg.type == MessageType.CMD_EXTRACT_EVIDENCE:
                    response = handle_extract_evidence(msg, extractor, cache_manager)
                
                elif msg.type == MessageType.CMD_PARSE_TABLE:
                     logger.warning("CMD_PARSE_TABLE not fully implemented yet")
                     response = create_error_message(msg.msg_id, ErrorCodes.E_PROC_001, "Table parsing not implemented")
                
                elif msg.type == MessageType.CMD_SHUTDOWN:
                    logger.info("🛑 Shutdown command received")
                    extractor.close()
                    response = create_message(MessageType.RES_ACK, {"status": "shutting_down"})
                    send_response(response)
                    break
                
                elif msg.type == MessageType.CMD_PING:
                    response = create_message(MessageType.RES_PONG, {})
                
                else:
                     logger.warning(f"Unknown command type: {msg.type}")

            except Exception as e:
                logger.error(f"❌ Processing error: {e}")
                logger.error(traceback.format_exc())
                
                response = create_error_message(
                    request_id=msg.msg_id,
                    error_code=ErrorCodes.E_SYS_001,
                    message=str(e),
                    details={"traceback": traceback.format_exc()}
                )

            # 4. Send response via stdout
            if response:
                send_response(response)

        except KeyboardInterrupt:
            logger.info("⚠️ Worker interrupted by user (Ctrl+C)")
            break
        
        except Exception as e:
            logger.critical(f"💥 Critical loop error: {e}")
            logger.critical(traceback.format_exc())
            break

    logger.info("👋 Worker shutdown complete")


def handle_handshake(msg: IpcMessage) -> IpcMessage:
    """Handle handshake request"""
    import sys
    
    payload = {
        "worker_pid": os.getpid(),
        "docling_version": "2.0.0-real",
        "python_version": f"{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}",
        "capabilities_supported": ["ocr", "table_extraction", "evidence_extraction"],
        "max_memory_mb": 1536,
        "status": "ready"
    }
    
    logger.info(f"✅ Handshake successful: PID {payload['worker_pid']}")
    return create_message(MessageType.RES_HANDSHAKE, payload)


def handle_extract_evidence(msg: IpcMessage, extractor: Any, cache: Any) -> IpcMessage:
    """Handle evidence extraction request with Caching"""
    import base64
    import time
    
    # Parse payload
    payload = ExtractEvidencePayload(**msg.payload)
    
    logger.info(f"🖼️  Extracting evidence from: {payload.file_path}")
    
    start_time = time.time()
    
    # 1. Generate Cache Key
    bbox_tuple = (payload.bbox.x, payload.bbox.y, payload.bbox.width, payload.bbox.height)
    cache_key = cache.generate_key(payload.file_path, payload.page_index, bbox_tuple, payload.dpi)
    
    # 2. Check Cache
    cache_status = CacheHit.MISS
    cached = cache.get(cache_key)
    
    b64_data = ""
    dims = (0, 0)
    fmt = payload.output_format or "jpeg"
    
    if cached:
        image_bytes, cached_dims, cached_fmt = cached
        b64_data = base64.b64encode(image_bytes).decode('utf-8')
        dims = cached_dims
        cache_status = CacheHit.DISK # SQLite is disk cache
        logger.info("   👉 Cache HIT (Disk)")
    else:
        logger.info("   👉 Cache MISS - Rendering...")
        # 3. Extract Real
        try:
            b64_data, dims = extractor.extract_with_fitz(
                file_path=payload.file_path,
                page_index=payload.page_index,
                bbox=bbox_tuple,
                dpi=payload.dpi,
                format=fmt,
                quality=payload.quality or 85
            )
            
            # 4. Save to Cache
            img_bytes = base64.b64decode(b64_data)
            cache.put(cache_key, payload.file_path, payload.page_index, bbox_tuple, 
                      img_bytes, dims, payload.dpi, fmt)
            
        except Exception as e:
             logger.error(f"Extraction failed: {e}")
             raise e

    duration_ms = int((time.time() - start_time) * 1000)
    
    # Add metadata
    metadata = {
        "extraction_time_ms": duration_ms,
        "cache_status": cache_status
    }
    
    return create_success_message(
        request_id=msg.msg_id,
        data={
            "image_base64": b64_data,
            "dimensions": list(dims),
            "format": fmt
        },
        metadata=metadata
    )


def send_response(msg: IpcMessage):
    """Send IPC message to stdout"""
    try:
        json_str = msg.to_json()
        sys.stdout.write(json_str + "\n")
        sys.stdout.flush()
        logger.info(f"📤 Sent: {msg.type}")
    except Exception as e:
        logger.error(f"❌ Failed to send response: {e}")


if __name__ == "__main__":
    main()
