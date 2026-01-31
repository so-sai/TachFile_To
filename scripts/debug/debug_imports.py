
try:
    from docling.document_converter import DocumentConverter
    print("Import DocumentConverter: OK")
except ImportError as e:
    print(f"Import DocumentConverter: FAIL - {e}")

try:
    from docling.datamodel.pipeline_options import PdfPipelineOptions
    print("Import PdfPipelineOptions: OK")
except ImportError as e:
    print(f"Import PdfPipelineOptions: FAIL - {e}")
