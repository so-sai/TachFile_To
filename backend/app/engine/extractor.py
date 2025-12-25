"""
Evidence Extraction Engine
Uses Docling v2 for PDF processing
"""

import logging
import time
from pathlib import Path
from typing import Dict, Any, List, Tuple

# Logger setup (Write to Stderr only to keep Stdout clean)
logger = logging.getLogger("worker.engine")


class EvidenceExtractor:
    """PDF Evidence Extraction Engine using Docling"""
    
    def __init__(self):
        logger.info("🔧 Initializing Docling Engine v2...")
        start = time.time()
        
        # TODO: Initialize Docling DocumentConverter
        # For now, mock initialization
        # from docling.document_converter import DocumentConverter
        # self.converter = DocumentConverter()
        self.converter = None  # Mock
        
        duration = time.time() - start
        logger.info(f"✅ Docling initialized in {duration:.2f}s")

    def extract_evidence(
        self, 
        file_path: str, 
        page_index: int, 
        bbox: Tuple[float, float, float, float], 
        dpi: int
    ) -> Dict[str, Any]:
        """
        Extract cropped image evidence from PDF page.
        
        Args:
            file_path: Absolute path to PDF
            page_index: 0-indexed page number
            bbox: (x, y, width, height) in PDF points
            dpi: Resolution (72, 150, or 300)
            
        Returns:
            Dict with keys: base64, width, height, mime_type
            
        Note:
            Current implementation returns MOCK DATA for IPC flow testing.
            Real implementation will use PyMuPDF (fitz) or pdf2image for cropping.
        """
        logger.info(f"📄 Processing Evidence: {file_path} (Page {page_index}, DPI {dpi})")
        
        # Validate file existence
        path = Path(file_path)
        if not path.exists():
            raise FileNotFoundError(f"File not found: {file_path}")
        
        # Validate bbox
        x, y, width, height = bbox
        if width <= 0 or height <= 0:
            raise ValueError(f"Invalid bbox dimensions: {bbox}")
        
        # --- MOCK IMPLEMENTATION ---
        # Simulate processing time
        time.sleep(0.1)
        
        # Generate dummy base64 (JPEG header)
        dummy_base64 = "/9j/4AAQSkZJRgABAQEAAAAAAAD/2wBDAAYEBQYFBAYGBQYHBwYIChAKCgkJChQODwwQFxQYGBcU"
        
        logger.info(f"✅ Evidence extracted: {int(width)}x{int(height)}px")
        
        return {
            "base64": dummy_base64,
            "width": int(width),
            "height": int(height),
            "mime_type": "image/jpeg"
        }

    def parse_table(
        self, 
        file_path: str, 
        page_index: int,
        hint_bbox: Tuple[float, float, float, float] | None = None
    ) -> Dict[str, Any]:
        """
        Parse table structure from PDF using Docling.
        
        Args:
            file_path: Absolute path to PDF
            page_index: 0-indexed page number
            hint_bbox: Optional bounding box hint for table location
            
        Returns:
            Dict with keys: rows (List[List[str]]), confidence (float)
        """
        logger.info(f"📊 Parsing Table: {file_path} (Page {page_index})")
        
        # Validate file
        path = Path(file_path)
        if not path.exists():
            raise FileNotFoundError(f"File not found: {file_path}")
        
        # --- MOCK IMPLEMENTATION ---
        # TODO: Use Docling when ready
        # result = self.converter.convert(file_path)
        # Extract tables from result.document.tables
        
        time.sleep(0.2)
        
        # Return mock table data
        mock_table = {
            "rows": [
                ["STT", "Vật liệu", "Đơn vị", "Khối lượng"],
                ["1", "Thép D10", "kg", "1234.56"],
                ["2", "Bê tông M250", "m³", "120.5"]
            ],
            "confidence": 0.92,
            "detected_columns": 4
        }
        
        logger.info(f"✅ Table parsed: {len(mock_table['rows'])} rows, confidence: {mock_table['confidence']:.2f}")
        
        return mock_table
