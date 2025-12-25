import sqlite3
import hashlib
import json
import time
import os
from pathlib import Path
from typing import Optional, Tuple, Any

class EvidenceCache:
    def __init__(self, db_path: str = "data/evidence_cache.db"):
        self.db_path = db_path
        self._ensure_db()

    def _ensure_db(self):
        """Initialize SQLite database and schema."""
        Path(self.db_path).parent.mkdir(parents=True, exist_ok=True)
        
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        # Enable WAL mode for concurrency
        cursor.execute("PRAGMA journal_mode=WAL;")
        
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS evidence_cache (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                cache_key TEXT UNIQUE NOT NULL,
                file_hash TEXT NOT NULL,
                page INTEGER NOT NULL,
                bbox TEXT NOT NULL,
                image_data BLOB NOT NULL,
                image_format TEXT DEFAULT 'jpeg',
                dimensions TEXT,
                dpi INTEGER DEFAULT 150,
                created_at INTEGER,
                last_accessed INTEGER,
                access_count INTEGER DEFAULT 1
            )
        """)
        
        # Create indexes
        cursor.execute("CREATE INDEX IF NOT EXISTS idx_cache_key ON evidence_cache(cache_key)")
        cursor.execute("CREATE INDEX IF NOT EXISTS idx_last_accessed ON evidence_cache(last_accessed)")
        
        conn.commit()
        conn.close()

    def generate_key(self, file_path: str, page: int, bbox: tuple, dpi: int) -> str:
        """Generate unique cache key."""
        # For MVP, we use file path as hash proxy. In prod, use real file hash.
        # Format: MD5(path|page|x,y,w,h|dpi)
        raw = f"{file_path}|{page}|{bbox}|{dpi}".encode('utf-8')
        return hashlib.md5(raw).hexdigest()

    def get(self, key: str) -> Optional[Tuple[bytes, Tuple[int, int], str]]:
        """
        Retrieve from cache.
        Returns: (image_bytes, (w, h), format) or None
        """
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        cursor.execute("""
            SELECT image_data, dimensions, image_format 
            FROM evidence_cache 
            WHERE cache_key = ?
        """, (key,))
        
        row = cursor.fetchone()
        
        if row:
            # Update access stats asynchronously (fire and forget in WAL mode ideally, but here sync is fine for MVP)
            cursor.execute("""
                UPDATE evidence_cache 
                SET last_accessed = ?, access_count = access_count + 1 
                WHERE cache_key = ?
            """, (int(time.time() * 1000), key))
            conn.commit()
            
            dims = json.loads(row[1])
            return row[0], (dims[0], dims[1]), row[2]
            
        conn.close()
        return None

    def put(self, key: str, file_path: str, page: int, bbox: tuple, 
            image_data: bytes, dimensions: Tuple[int, int], dpi: int, fmt: str = "jpeg"):
        """Save to cache."""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        now = int(time.time() * 1000)
        dims_json = json.dumps(list(dimensions))
        bbox_json = json.dumps(list(bbox))
        
        try:
            cursor.execute("""
                INSERT OR REPLACE INTO evidence_cache 
                (cache_key, file_hash, page, bbox, image_data, image_format, dimensions, dpi, created_at, last_accessed)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """, (key, file_path, page, bbox_json, image_data, fmt, dims_json, dpi, now, now))
            conn.commit()
        except sqlite3.Error as e:
            print(f"Cache write error: {e}")
        finally:
            conn.close()

    def prune(self, max_size_mb: int = 500, max_age_days: int = 30):
        """Prune old items to stay within limits."""
        # Implementation skipped for MVP, but placeholder logic:
        # 1. DELETE FROM evidence_cache WHERE last_accessed < ...
        # 2. Check DB size, delete oldest if > max_size_mb
        pass
