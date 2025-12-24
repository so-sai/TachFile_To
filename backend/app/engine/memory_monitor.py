"""
Memory Watchdog for Python Worker
Monitors RAM usage and kills process if limits exceeded

Based on backend_worker_v1.md specification.
"""
import threading
import time
import gc
import sys
import os
import json
from typing import Optional, Callable
import signal

try:
    import psutil
    HAS_PSUTIL = True
except ImportError:
    HAS_PSUTIL = False
    print("[Watchdog] WARNING: psutil not installed, memory monitoring disabled", 
          file=sys.stderr)


class MemoryWatchdog:
    """
    Background thread that monitors memory usage.
    
    - Soft limit: Trigger garbage collection
    - Hard limit: Kill process to prevent system freeze
    """
    
    def __init__(
        self,
        soft_limit_mb: int = 1024,   # 1GB - trigger GC
        hard_limit_mb: int = 1536,   # 1.5GB - kill process
        check_interval: float = 1.0,  # seconds
        on_soft_limit: Optional[Callable[[], None]] = None,
        on_hard_limit: Optional[Callable[[], None]] = None,
    ):
        self.soft_limit_mb = soft_limit_mb
        self.hard_limit_mb = hard_limit_mb
        self.check_interval = check_interval
        self.on_soft_limit = on_soft_limit
        self.on_hard_limit = on_hard_limit
        
        self.running = False
        self.thread: Optional[threading.Thread] = None
        self.process = psutil.Process(os.getpid()) if HAS_PSUTIL else None
        
        # Statistics
        self.soft_limit_hits = 0
        self.hard_limit_hits = 0
        self.gc_collections = 0
        self.peak_memory_mb = 0.0
        self.start_time = time.time()
    
    def get_memory_usage_mb(self) -> float:
        """Get current RSS memory usage in MB"""
        if not HAS_PSUTIL or not self.process:
            return 0.0
        
        try:
            return self.process.memory_info().rss / 1024 / 1024
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            return 0.0
    
    def _send_oom_warning(self, memory_mb: float):
        """Send OOM warning to Rust parent via stdout"""
        try:
            warning = {
                "type": "lifecycle",
                "event": "oom_warning",
                "memory_mb": round(memory_mb, 1),
                "limit_mb": self.hard_limit_mb,
                "pid": os.getpid()
            }
            print(json.dumps(warning), file=sys.stdout, flush=True)
            time.sleep(0.1)  # Give Rust time to read
        except Exception as e:
            print(f"[Watchdog] Failed to send OOM warning: {e}", file=sys.stderr)
    
    def _watchdog_loop(self):
        """Main watchdog loop"""
        print(f"[Watchdog] Started (soft: {self.soft_limit_mb}MB, "
              f"hard: {self.hard_limit_mb}MB)", file=sys.stderr)
        
        consecutive_soft_hits = 0
        
        while self.running:
            try:
                memory_mb = self.get_memory_usage_mb()
                
                # Track peak memory
                if memory_mb > self.peak_memory_mb:
                    self.peak_memory_mb = memory_mb
                
                # Check hard limit (immediate suicide)
                if memory_mb > self.hard_limit_mb:
                    self.hard_limit_hits += 1
                    print(f"[Watchdog] CRITICAL: Memory {memory_mb:.1f}MB > "
                          f"{self.hard_limit_mb}MB - Killing process", 
                          file=sys.stderr)
                    
                    # Callback before death
                    if self.on_hard_limit:
                        try:
                            self.on_hard_limit()
                        except:
                            pass
                    
                    # Send warning to Rust
                    self._send_oom_warning(memory_mb)
                    
                    # Kill ourselves - use SIGTERM on Windows
                    if sys.platform == "win32":
                        os._exit(137)
                    else:
                        os.kill(os.getpid(), signal.SIGTERM)
                    return
                
                # Check soft limit (trigger GC)
                elif memory_mb > self.soft_limit_mb:
                    consecutive_soft_hits += 1
                    self.soft_limit_hits += 1
                    
                    # Only GC if we've hit soft limit 3+ times in a row
                    if consecutive_soft_hits >= 3:
                        print(f"[Watchdog] WARNING: Memory {memory_mb:.1f}MB > "
                              f"{self.soft_limit_mb}MB - Forcing GC", 
                              file=sys.stderr)
                        
                        # Callback
                        if self.on_soft_limit:
                            try:
                                self.on_soft_limit()
                            except:
                                pass
                        
                        # Aggressive garbage collection
                        collected = gc.collect(generation=2)
                        self.gc_collections += 1
                        
                        # Clear cyclic garbage
                        if hasattr(gc, 'garbage'):
                            gc.garbage.clear()
                        
                        print(f"[Watchdog] GC collected {collected} objects", 
                              file=sys.stderr)
                        
                        consecutive_soft_hits = 0
                else:
                    consecutive_soft_hits = 0
                
                time.sleep(self.check_interval)
                
            except Exception as e:
                print(f"[Watchdog] Error in watchdog loop: {e}", 
                      file=sys.stderr)
                time.sleep(self.check_interval)
    
    def start(self):
        """Start the watchdog thread"""
        if self.running:
            return
        
        if not HAS_PSUTIL:
            print("[Watchdog] Cannot start - psutil not available", 
                  file=sys.stderr)
            return
        
        self.running = True
        self.start_time = time.time()
        self.thread = threading.Thread(
            target=self._watchdog_loop,
            name="MemoryWatchdog",
            daemon=True  # Daemon thread dies when main thread exits
        )
        self.thread.start()
    
    def stop(self):
        """Stop the watchdog thread"""
        self.running = False
        if self.thread and self.thread.is_alive():
            self.thread.join(timeout=2.0)
        print(f"[Watchdog] Stopped. Stats: soft_hits={self.soft_limit_hits}, "
              f"gc_runs={self.gc_collections}, peak_ram={self.peak_memory_mb:.1f}MB",
              file=sys.stderr)
    
    def get_stats(self) -> dict:
        """Get watchdog statistics"""
        return {
            "current_memory_mb": round(self.get_memory_usage_mb(), 1),
            "peak_memory_mb": round(self.peak_memory_mb, 1),
            "soft_limit_mb": self.soft_limit_mb,
            "hard_limit_mb": self.hard_limit_mb,
            "soft_limit_hits": self.soft_limit_hits,
            "hard_limit_hits": self.hard_limit_hits,
            "gc_collections": self.gc_collections,
            "uptime_seconds": round(time.time() - self.start_time, 1),
            "is_running": self.running
        }


# Singleton instance for easy access
_watchdog_instance: Optional[MemoryWatchdog] = None


def get_watchdog() -> Optional[MemoryWatchdog]:
    """Get the global watchdog instance"""
    return _watchdog_instance


def init_watchdog(
    soft_limit_mb: int = 1024,
    hard_limit_mb: int = 1536
) -> MemoryWatchdog:
    """Initialize and start the global watchdog"""
    global _watchdog_instance
    
    if _watchdog_instance is not None:
        _watchdog_instance.stop()
    
    _watchdog_instance = MemoryWatchdog(
        soft_limit_mb=soft_limit_mb,
        hard_limit_mb=hard_limit_mb
    )
    _watchdog_instance.start()
    
    return _watchdog_instance
