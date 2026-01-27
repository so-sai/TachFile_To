#!/usr/bin/env python3
"""Quick test for real MuPDF integration"""

import sys
import os

# Add built library to path
sys.path.insert(0, "E:/DEV/elite_9_VN-ecosystem/app-tool-TachFileTo/target/release")

try:
    import elite_pdf

    # Test with a real PDF file
    test_pdf = "E:/DEV/elite_9_VN-ecosystem/app-tool-TachFileTo/test/pdf/BoQ 16052022-REV 3.pdf"

    if not os.path.exists(test_pdf):
        print("Test PDF not found, using mock test...")
        # Test with dummy path to see if we can at least create the object
        try:
            doc = elite_pdf.EliteDocument("dummy.pdf")
            print("Should have failed with invalid path")
        except Exception as e:
            print(f"Correctly failed with dummy: {e}")
    else:
        print(f"Testing with: {test_pdf}")
        doc = elite_pdf.EliteDocument(test_pdf)
        pages = doc.count_pages()
        print(f"SUCCESS: Real page count = {pages}")

        if pages > 0 and pages != 112:  # 112 was our mock value
            print("MuPDF integration is REAL and working!")
        else:
            print("Unexpected page count")

except ImportError as e:
    print(f"Failed to import elite_pdf: {e}")
except Exception as e:
    print(f"Test failed: {e}")
