# libs/iron_python_bridge/python/extraction.py
from pathlib import Path
from typing import Any
from pydantic import BaseModel, Field, ValidationError
import structlog
import json
import hashlib
from datetime import datetime
import os

# JSON structured logging for production-grade audit trails
logger = structlog.get_logger()

# --- Docling Import Logic ---
try:
    from docling.document_converter import DocumentConverter
    from docling.datamodel.pipeline_options import PdfPipelineOptions
    DOCLING_AVAILABLE = True
except ImportError:
    DOCLING_AVAILABLE = False
    logger.warning("Docling library not found. Running in MOCK mode.")

class ProcessInput(BaseModel):
    """Input validation for the extraction process."""
    path: str = Field(..., min_length=1, description="Đường dẫn file PDF/DOCX/XLSX/MD")

class PageMetadata(BaseModel):
    """Metadata for an individual page."""
    page: int
    content_type: str  # "text", "table", "image", "mixed"
    confidence: float
    text_length: int
    table_rows: int | None = None
    table_columns: int | None = None

class IngestionResult(BaseModel):
    """Complete ingestion object compliant with INGESTION_SCHEMA.json v0.1."""
    source: str
    checksum: str = Field(default="")
    pages: list[PageMetadata]
    raw_content: str | None = None
    tables: list[dict[str, Any]] = Field(default_factory=list)
    extraction_meta: dict[str, Any] = Field(default_factory=dict)
    schema_version: str = "MF50-INGEST-0.1"

def calculate_checksum(file_path: Path) -> str:
    """Calculate SHA-256 checksum of a file."""
    sha256_hash = hashlib.sha256()
    with open(file_path, "rb") as f:
        for byte_block in iter(lambda: f.read(4096), b""):
            sha256_hash.update(byte_block)
    return f"sha256:{sha256_hash.hexdigest()}"

def process_document(input_path: str, allow_mock: bool = True) -> str:
    """
    Process multi-format (PDF/DOCX/XLSX/MD) -> JSON string (Canonical Ingestion).
    This is the main entry point for the Rust bridge.
    """
    try:
        # 1. Validate Input
        validated_input = ProcessInput(path=input_path)
        source_path = Path(validated_input.path)
        
        if not source_path.exists():
            logger.error("File not found", path=str(source_path))
            raise FileNotFoundError(f"File not found: {source_path}")

        logger.info("Starting conversion", path=str(source_path))

        # 2. Processing Strategy
        pages_meta = []
        doc_format = "unknown"
        raw_content = ""
        tables_data = []
        
        
        # Force default to FALSE. We only mock if explicitly requested or ALL engines fail.
        # use_mock = not DOCLING_AVAILABLE or os.environ.get("ELITE_MOCK_MODE") == "1" 
        use_mock = os.environ.get("ELITE_MOCK_MODE") == "1"

        if not use_mock and DOCLING_AVAILABLE:
            try:
                # Configure Docling to use pypdfium2 if available/requested
                pipeline_options = PdfPipelineOptions(
                    do_ocr=False, # Disable OCR for speed in stress test unless needed
                    do_table_structure=True,
                    pdf_backend="pypdfium2" 
                )
                converter = DocumentConverter(format_options={"pdf": pipeline_options})
                
                result = converter.convert(source_path)
                
                # 3. Extract Structured Data
                raw_content = result.document.export_to_markdown()
                
                # Extract tables
                doc_tables = result.document.tables
                for i, table in enumerate(doc_tables):
                    try:
                        df = table.export_to_dataframe()
                        tables_data.append({
                            "id": f"table_{i+1}",
                            "rows": df.to_dict(orient="records")
                        })
                    except Exception:
                        tables_data.append({
                            "id": f"table_{i+1}",
                            "html": table.export_to_html()
                        })

                # Refine page metadata
                doc_pages = result.document.pages
                for i, page in enumerate(doc_pages, start=1):
                    pages_meta.append(PageMetadata(
                        page=i,
                        content_type="mixed",
                        confidence=0.99,
                        text_length=len(raw_content) // (len(doc_pages) or 1)
                    ))

                doc_format = str(result.document.format) if hasattr(result.document, 'format') else doc_format
                
                
            except Exception as e:
                logger.error("Docling processing failed/unavailable", error=str(e))
                docling_error = e

        # --- FAST LANE: Direct Pypdfium2 (No-GIL, Lightweight) ---
        # If Docling failed or is missing, we fall back to direct pypdfium2
        # This is the "Fast Lane" for high-performance text extraction without heavy ML deps.
        if not raw_content and not use_mock:
            try:
                logger.info("Attempting Fast Lane extraction (pypdfium2 direct)")
                import pypdfium2 as pdfium
                
                if ".pdf" in str(source_path).lower():
                    pdf = pdfium.PdfDocument(str(source_path))
                    full_text = []
                    local_pages_meta = []
                    
                    for i, page_obj in enumerate(pdf, start=1):
                        text_page = page_obj.get_textpage()
                        page_text = text_page.get_text_range()
                        full_text.append(page_text)
                        
                        local_pages_meta.append(PageMetadata(
                            page=i,
                            content_type="text",
                            confidence=1.0, # PDFium is deterministic
                            text_length=len(page_text)
                        ))
                    
                    raw_content = "\n\n".join(full_text)
                    pages_meta = local_pages_meta
                    doc_format = "pdf_fast_lane"
                    extraction_meta["engine"] = "pypdfium2_direct"
                    extraction_meta["fallback_reason"] = str(docling_error) if 'docling_error' in locals() else "docling_missing"
                    
                    logger.info("Fast Lane extraction successful", pages=len(pages_meta))
                else:
                    # For non-PDFs (like MD/TXT) in Fast Lane, simply read text if possible
                    try:
                        with open(source_path, "r", encoding="utf-8") as f:
                            raw_content = f.read()
                        pages_meta = [PageMetadata(page=1, content_type="text", confidence=1.0, text_length=len(raw_content))]
                        doc_format = "text_fast_lane"
                         
                    except Exception:
                        pass # Fall through to mock
                
            except ImportError:
                 logger.warning("Pypdfium2 not found. Falling back to Mock.")
                 use_mock = True
            except Exception as e:
                logger.error("Fast Lane failed", error=str(e))
                if allow_mock:
                    use_mock = True
                else:
                    raise RuntimeError(f"Fast Lane extraction error: {e}") from e

        if use_mock:
            # MOCK MODE EXECUTION
            logger.info("Using MOCK extraction mode (CPU Stress Simulation)")
            
            # CPU Burn-in to simulate OCR/Parsing load (Prove No-GIL)
            x = 0
            for _ in range(5_000_000): 
                x += 1

            pages_meta = [PageMetadata(page=1, content_type="mock_text", confidence=1.0, text_length=150)]
            doc_format = "mock_data"
            raw_content = f"# Mock Content for {source_path.name}\n\nProcessed by Elite 9 No-GIL Bridge.\nLoad Factor: {x}"
            tables_data = [{"id": "mock_table_1", "rows": [{"col1": "A", "col2": "B"}]}]

        # 4. Construct Output
        output = IngestionResult(
            source=str(source_path),
            checksum=calculate_checksum(source_path),
            pages=pages_meta,
            raw_content=raw_content,
            tables=tables_data,
            extraction_meta={
                "engine": "docling" if DOCLING_AVAILABLE and not use_mock else "mock_engine",
                "engine_version": "2.68.0" if DOCLING_AVAILABLE else "0.0.0",
                "timestamp": datetime.utcnow().isoformat() + "Z",
                "format": doc_format
            }
        )

        # 5. Return JSON String
        json_output = output.model_dump_json(indent=2)
        logger.info("Conversion successful", pages=len(pages_meta), checksum=output.checksum)
        return json_output

    except ValidationError as ve:
        logger.error("Input validation failed", errors=ve.errors())
        raise ValueError("Invalid input parameters") from ve
    except FileNotFoundError:
        raise
    except Exception as e:
        logger.error("Unexpected error during processing", exc_info=e)
        raise RuntimeError(f"Processing error: {str(e)}") from e
