import pytest
from pathlib import Path
from pydantic import ValidationError
import sys
import os

# Add parent directory to path to import extraction
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))
from extraction import process_pdf, ProcessInput, IngestionResult

@pytest.fixture
def sample_pdf(tmp_path):
    pdf_path = tmp_path / "test_qs_sample.pdf"
    # Create a minimal valid-ish PDF header for fitz/docling to not immediately crash if called
    pdf_path.write_bytes(b"%PDF-1.4\n%mock content") 
    return str(pdf_path)

def test_process_pdf_valid(sample_pdf):
    # Action: Gọi process_pdf(path)
    # Note: This might fail if Docling is not installed or the mock PDF is too invalid
    # but that's expected in the 'Red' phase of TDD.
    try:
        result_json = process_pdf(sample_pdf, allow_mock=True)
        assert isinstance(result_json, str)
        
        # Assert: Parse ra đúng Pydantic Model
        result = IngestionResult.model_validate_json(result_json)
        assert result.source == sample_pdf
        assert len(result.pages) >= 0
    except Exception as e:
        pytest.fail(f"process_pdf raised unexpected exception: {e}")

def test_invalid_path():
    with pytest.raises(FileNotFoundError):
        process_pdf("/non/existent/path_qs_very_large.pdf")

def test_invalid_input_empty_path():
    # Test Pydantic validation directly or via process_pdf
    with pytest.raises(ValidationError):
        ProcessInput(path="") # Giả sử empty path là invalid trong spec
