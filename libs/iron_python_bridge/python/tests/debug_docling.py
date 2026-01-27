from pathlib import Path
import sys
import os

# Add parent directory to path to import extraction
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))
from extraction import process_pdf

def debug_process():
    dummy_pdf = Path("debug_test.pdf")
    dummy_pdf.write_bytes(b"%PDF-1.4\n%mock content")
    
    try:
        print(f"Testing process_pdf with {dummy_pdf}...")
        result = process_pdf(str(dummy_pdf))
        print("Success!")
        print(result)
    except Exception as e:
        print("\n--- ERROR DETECTED ---")
        import traceback
        traceback.print_exc()
    finally:
        if dummy_pdf.exists():
            dummy_pdf.unlink()

if __name__ == "__main__":
    debug_process()
