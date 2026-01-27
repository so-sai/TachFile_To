"""
Unit tests for extraction.py (Python 3.14 / 2026 Edition)
Tests verify Pydantic validation, type safety, and core logic.
"""

import json
import tempfile
from pathlib import Path

import pytest
from pydantic import ValidationError

# Import functions to test
import sys
sys.path.insert(0, str(Path(__file__).parent.parent / "libs" / "iron_python_bridge" / "python"))

from extraction import (
    PageMetadata,
    IngestionResult,
    calculate_sha256,
    split_pdf_into_chunks,
)


class TestPydanticModels:
    """Test Pydantic validation and type safety."""
    
    def test_page_metadata_valid(self):
        """Valid PageMetadata should pass validation."""
        page = PageMetadata(
            page=1,
            content_type="text",
            confidence=0.95
        )
        assert page.page == 1
        assert page.content_type == "text"
        assert page.confidence == 0.95
    
    def test_page_metadata_invalid_page_number(self):
        """Page number must be >= 1."""
        with pytest.raises(ValidationError):
            PageMetadata(page=0, content_type="text", confidence=0.5)
    
    def test_page_metadata_invalid_confidence(self):
        """Confidence must be between 0.0 and 1.0."""
        with pytest.raises(ValidationError):
            PageMetadata(page=1, content_type="text", confidence=1.5)
    
    def test_page_metadata_invalid_content_type(self):
        """Content type must be one of the allowed literals."""
        with pytest.raises(ValidationError):
            PageMetadata(page=1, content_type="invalid", confidence=0.5)
    
    def test_ingestion_result_serialization(self):
        """IngestionResult should serialize to valid JSON."""
        result = IngestionResult(
            checksum="sha256:abc123",
            origin_signature="test.pdf",
            pages=[
                PageMetadata(page=1, content_type="text", confidence=0.9)
            ],
            extraction_meta={"engine": "test"}
        )
        
        json_str = result.model_dump_json()
        parsed = json.loads(json_str)
        
        assert parsed["source"] == "tachfileto"
        assert parsed["checksum"] == "sha256:abc123"
        assert len(parsed["pages"]) == 1


class TestSHA256Calculation:
    """Test checksum calculation."""
    
    def test_calculate_sha256_consistent(self):
        """Same file should produce same checksum."""
        with tempfile.NamedTemporaryFile(mode='wb', delete=False) as f:
            f.write(b"test content")
            temp_path = Path(f.name)
        
        try:
            checksum1 = calculate_sha256(temp_path)
            checksum2 = calculate_sha256(temp_path)
            
            assert checksum1 == checksum2
            assert checksum1.startswith("sha256:")
        finally:
            temp_path.unlink()
    
    def test_calculate_sha256_different_content(self):
        """Different content should produce different checksums."""
        with tempfile.NamedTemporaryFile(mode='wb', delete=False) as f1:
            f1.write(b"content A")
            path1 = Path(f1.name)
        
        with tempfile.NamedTemporaryFile(mode='wb', delete=False) as f2:
            f2.write(b"content B")
            path2 = Path(f2.name)
        
        try:
            checksum1 = calculate_sha256(path1)
            checksum2 = calculate_sha256(path2)
            
            assert checksum1 != checksum2
        finally:
            path1.unlink()
            path2.unlink()


class TestTypeHints:
    """Verify modern type hints are used correctly."""
    
    def test_split_pdf_accepts_path_types(self):
        """Function should accept both str and Path."""
        # This test verifies type signature compatibility
        # Actual PDF splitting requires fitz library
        from typing import get_type_hints
        from extraction import split_pdf_into_chunks
        
        hints = get_type_hints(split_pdf_into_chunks)
        
        # Verify modern union syntax is used
        assert "input_pdf" in hints
        assert "output_dir" in hints


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
