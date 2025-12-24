#!/usr/bin/env python3
"""
TachFileTo Python Worker - Heavy Computation Engine
Communicates via stdin/stdout JSON Lines protocol

Based on backend_worker_v1.md specification.
"""
import sys
import json
import asyncio
import signal
import os
import time
import traceback
from typing import Optional, Dict, Any

# Add parent to path for imports
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from app.protocol import (
    WorkerRequest,
    WorkerResponse,
    ReadyMessage,
    CommandType,
    ErrorType,
)
from app.engine.memory_monitor import init_watchdog, get_watchdog


# Version of this worker
VERSION = "0.1.0"


class PythonWorker:
    """
    Main worker class that handles IPC with Rust.
    
    Lifecycle:
    1. Initialize (load heavy models)
    2. Send ready signal
    3. Main loop: read stdin -> process -> write stdout
    4. Shutdown on signal
    """
    
    def __init__(self):
        self.running = True
        self.request_count = 0
        self.error_count = 0
        self.start_time = time.time()
        self.worker_id = f"worker_{os.getpid()}"
        
        # Will be initialized lazily
        self._extractor = None
    
    def setup_signal_handlers(self):
        """Handle graceful shutdown"""
        def signal_handler(sig, frame):
            print(f"[Worker] Received signal {sig}, shutting down...", 
                  file=sys.stderr)
            self.running = False
        
        # Windows uses different signals
        signal.signal(signal.SIGINT, signal_handler)
        signal.signal(signal.SIGTERM, signal_handler)
        
        if sys.platform == "win32":
            try:
                signal.signal(signal.SIGBREAK, signal_handler)
            except (AttributeError, ValueError):
                pass
    
    async def initialize(self):
        """Initialize heavy components once"""
        print("[Worker] Initializing...", file=sys.stderr)
        
        # Start memory watchdog
        ram_limit = int(os.environ.get("TACH_WORKER_RAM_LIMIT_MB", "1024"))
        init_watchdog(
            soft_limit_mb=ram_limit,
            hard_limit_mb=int(ram_limit * 1.5)
        )
        
        # TODO: Initialize Docling here (expensive operation)
        # self._extractor = EvidenceExtractor()
        # await self._extractor.initialize()
        
        print("[Worker] Initialization complete", file=sys.stderr)
    
    def send_ready(self):
        """Send ready signal to Rust parent process"""
        # Detect capabilities
        capabilities = ["basic"]
        
        try:
            import docling
            capabilities.append("docling")
        except ImportError:
            pass
        
        try:
            import psutil
            capabilities.append("memory_monitor")
        except ImportError:
            pass
        
        ready_msg = ReadyMessage(
            version=VERSION,
            pid=os.getpid(),
            capabilities=capabilities,
            worker_id=self.worker_id
        )
        
        self._write_message(ready_msg.model_dump())
        print(f"[Worker] Ready signal sent (capabilities: {capabilities})", 
              file=sys.stderr)
    
    def _write_message(self, data: Dict[str, Any]):
        """Write JSON line to stdout (IPC channel to Rust)"""
        try:
            json_line = json.dumps(data, ensure_ascii=False, default=str)
            sys.stdout.write(json_line + "\n")
            sys.stdout.flush()
        except Exception as e:
            print(f"[Worker] Failed to write message: {e}", file=sys.stderr)
    
    async def handle_command(self, request: WorkerRequest) -> WorkerResponse:
        """Route command to appropriate handler"""
        start_time = time.time()
        
        try:
            if request.cmd == CommandType.HEALTH_CHECK:
                result = await self._handle_health_check()
            
            elif request.cmd == CommandType.FORCE_GC:
                result = await self._handle_force_gc()
            
            elif request.cmd == CommandType.EXTRACT_EVIDENCE:
                result = await self._handle_extract_evidence(request.payload)
            
            elif request.cmd == CommandType.SHUTDOWN:
                self.running = False
                result = {"status": "shutting_down"}
            
            else:
                raise ValueError(f"Unknown command: {request.cmd}")
            
            duration_ms = (time.time() - start_time) * 1000
            
            # Get memory stats if available
            peak_ram_mb = None
            watchdog = get_watchdog()
            if watchdog:
                peak_ram_mb = watchdog.get_stats().get("current_memory_mb")
            
            return WorkerResponse.success(
                req_id=request.id,
                data=result,
                duration_ms=duration_ms,
                peak_ram_mb=peak_ram_mb
            )
            
        except asyncio.TimeoutError:
            self.error_count += 1
            return WorkerResponse.error(
                req_id=request.id,
                error_type=ErrorType.TIMEOUT_EXCEEDED,
                message="Operation timed out"
            )
        
        except FileNotFoundError as e:
            self.error_count += 1
            return WorkerResponse.error(
                req_id=request.id,
                error_type=ErrorType.FILE_NOT_FOUND,
                message=str(e)
            )
        
        except Exception as e:
            self.error_count += 1
            tb = traceback.format_exc()
            print(f"[Worker] Error handling command: {e}\n{tb}", file=sys.stderr)
            
            return WorkerResponse.error(
                req_id=request.id,
                error_type=ErrorType.UNKNOWN,
                message=str(e),
                traceback=tb[-500:]  # Last 500 chars of traceback
            )
    
    async def _handle_health_check(self) -> dict:
        """Handle health check command"""
        watchdog = get_watchdog()
        watchdog_stats = watchdog.get_stats() if watchdog else {}
        
        return {
            "status": "healthy",
            "worker_id": self.worker_id,
            "uptime_seconds": time.time() - self.start_time,
            "requests_processed": self.request_count,
            "error_count": self.error_count,
            "memory_mb": watchdog_stats.get("current_memory_mb", 0),
            "peak_memory_mb": watchdog_stats.get("peak_memory_mb", 0),
        }
    
    async def _handle_force_gc(self) -> dict:
        """Force garbage collection"""
        import gc
        
        before = get_watchdog().get_memory_usage_mb() if get_watchdog() else 0
        collected = gc.collect(generation=2)
        after = get_watchdog().get_memory_usage_mb() if get_watchdog() else 0
        
        return {
            "collected_objects": collected,
            "memory_before_mb": round(before, 1),
            "memory_after_mb": round(after, 1),
            "freed_mb": round(before - after, 1)
        }
    
    async def _handle_extract_evidence(self, payload: dict) -> dict:
        """Extract evidence from PDF - PLACEHOLDER"""
        # TODO: Implement actual Docling extraction
        # This is a placeholder that returns dummy data
        
        file_path = payload.get("file_path", "")
        page_index = payload.get("page_index", 0)
        bbox = payload.get("bbox", [0, 0, 100, 100])
        dpi = payload.get("dpi", 150)
        
        print(f"[Worker] Extract evidence: {file_path} page={page_index} "
              f"bbox={bbox} dpi={dpi}", file=sys.stderr)
        
        # Check if file exists
        if not os.path.exists(file_path):
            raise FileNotFoundError(f"File not found: {file_path}")
        
        # TODO: Replace with actual extraction
        # For now, return placeholder
        return {
            "base64": "",  # Empty for now
            "width": 100,
            "height": 100,
            "mime_type": "image/jpeg",
            "placeholder": True
        }
    
    async def process_line(self, line: str):
        """Process a single line from stdin"""
        if not line.strip():
            return
        
        try:
            raw_data = json.loads(line.strip())
            request = WorkerRequest(**raw_data)
            
            response = await self.handle_command(request)
            self._write_message(response.model_dump())
            self.request_count += 1
            
        except json.JSONDecodeError as e:
            print(f"[Worker] Invalid JSON: {e}", file=sys.stderr)
            self._write_message({
                "req_id": "unknown",
                "status": "error",
                "error": {
                    "type": "parsing_failed",
                    "message": f"Invalid JSON: {e}"
                }
            })
    
    async def run_loop(self):
        """Main event loop - read from stdin, process, write to stdout"""
        loop = asyncio.get_event_loop()
        
        print("[Worker] Starting main loop...", file=sys.stderr)
        
        while self.running:
            try:
                # Read line with timeout
                try:
                    line = await asyncio.wait_for(
                        loop.run_in_executor(None, sys.stdin.readline),
                        timeout=0.5
                    )
                except asyncio.TimeoutError:
                    continue
                
                if line:
                    await self.process_line(line)
                elif line == "":
                    # EOF - parent closed pipe
                    print("[Worker] EOF detected, shutting down", file=sys.stderr)
                    self.running = False
                
            except Exception as e:
                print(f"[Worker] Error in main loop: {e}", file=sys.stderr)
                await asyncio.sleep(0.01)
    
    async def run(self):
        """Main entry point"""
        self.setup_signal_handlers()
        await self.initialize()
        self.send_ready()
        await self.run_loop()


async def main():
    """Entry point"""
    worker = PythonWorker()
    
    try:
        await worker.run()
    except KeyboardInterrupt:
        print("[Worker] Keyboard interrupt", file=sys.stderr)
    finally:
        # Cleanup
        watchdog = get_watchdog()
        if watchdog:
            watchdog.stop()
        
        print(f"[Worker] Shutdown complete. Processed {worker.request_count} requests, "
              f"{worker.error_count} errors", file=sys.stderr)


if __name__ == "__main__":
    asyncio.run(main())
