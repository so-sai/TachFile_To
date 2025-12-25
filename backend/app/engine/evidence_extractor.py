import base64
import io
import time
from typing import Optional, Tuple, Dict, Any

# Lazy import fitz to allow worker to start even if PyMuPDF is missing
try:
    import fitz
    from PIL import Image
    HAS_FITZ = True
except ImportError:
    fitz = None
    Image = None
    HAS_FITZ = False

class RealEvidenceExtractor:
    def __init__(self):
        # Cache for open fitz Documents to avoid re-opening file repeatedly
        # Key: file_path, Value: (fitz.Document, last_accessed_time)
        self._doc_cache: Dict[str, Any] = {}
        self._cache_size = 5
        self._cache_ttl = 600  # 10 minutes
        
        if not HAS_FITZ:
            # We don't raise error here, we raise it when extract is called
            pass

    def _get_document(self, file_path: str):
        """Get fitz document from cache or open new one."""
        if not HAS_FITZ:
            raise ImportError("PyMuPDF (fitz) is not installed. Cannot open PDF.")

        now = time.time()
        
        # Clean expired or overflow
        keys_to_remove = []
        for k, v in self._doc_cache.items():
            if now - v[1] > self._cache_ttl:
                v[0].close()
                keys_to_remove.append(k)
        for k in keys_to_remove:
            del self._doc_cache[k]

        if len(self._doc_cache) >= self._cache_size:
            # Simple eviction
            k = next(iter(self._doc_cache))
            self._doc_cache[k][0].close()
            del self._doc_cache[k]

        if file_path in self._doc_cache:
            self._doc_cache[file_path] = (self._doc_cache[file_path][0], now)
            return self._doc_cache[file_path][0]

        try:
            doc = fitz.open(file_path)
            self._doc_cache[file_path] = (doc, now)
            return doc
        except Exception as e:
            raise ValueError(f"Failed to open PDF: {e}")

    def extract_with_fitz(
        self, 
        file_path: str, 
        page_index: int, 
        bbox: Tuple[float, float, float, float], 
        dpi: int = 150, 
        format: str = "jpeg", 
        quality: int = 85
    ) -> Tuple[str, Tuple[int, int]]:
        """
        Render a specific bbox from a PDF page to an image.
        """
        if not HAS_FITZ:
            raise ImportError("PyMuPDF (fitz) is not installed.")

        doc = self._get_document(file_path)
        
        if page_index < 0 or page_index >= len(doc):
             raise ValueError(f"Page index {page_index} out of range (0-{len(doc)-1})")
             
        page = doc[page_index]
        
        # BBox format: [x, y, width, height] -> fitz.Rect(x0, y0, x1, y1)
        x, y, w, h = bbox
        rect = fitz.Rect(x, y, x + w, y + h)
        
        # Add small padding
        padding = 5
        rect.x0 = max(0, rect.x0 - padding)
        rect.y0 = max(0, rect.y0 - padding)
        rect.x1 = min(page.rect.x1, rect.x1 + padding)
        rect.y1 = min(page.rect.y1, rect.y1 + padding)

        # Zoom matrix for DPI
        zoom = dpi / 72.0
        mat = fitz.Matrix(zoom, zoom)
        
        # Render to pixmap
        pix = page.get_pixmap(matrix=mat, clip=rect, alpha=False)
        
        # Convert to bytes
        img_data = pix.tobytes(format)
        
        # Encode value
        b64_str = base64.b64encode(img_data).decode("utf-8")
        
        return b64_str, (pix.width, pix.height)

    def close(self):
        """Clean up all open documents."""
        if not HAS_FITZ:
            return
            
        for v in self._doc_cache.values():
            try:
                v[0].close()
            except:
                pass
        self._doc_cache.clear()
