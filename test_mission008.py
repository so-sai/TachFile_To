#!/usr/bin/env python3
"""
Mission 008 Test: PDF to Markdown Extraction
Test the new structured text extraction capabilities
"""

import sys
import os

sys.path.insert(
    0,
    os.path.join(
        os.path.dirname(__file__), "..", "libs", "elite_pdf", "target", "debug"
    ),
)

from elite_pdf import EliteDocument


def test_markdown_extraction():
    """Test PDF to Markdown extraction"""
    print("🛡️ Mission 008: PDF to Markdown Extraction Test")
    print("=" * 50)

    # Test file path (you'll need to provide a test PDF)
    test_pdf = r"E:\DEV\elite_9_VN-ecosystem\app-tool-TachFileTo\test_files\sample.pdf"

    if not os.path.exists(test_pdf):
        print(f"❌ Test file not found: {test_pdf}")
        print("Please place a test PDF at the above location")
        return False

    try:
        # 1. Open document
        print("📂 Opening PDF document...")
        doc = EliteDocument(test_pdf)
        page_count = doc.count_pages()
        print(f"✅ Document opened: {page_count} pages")

        # 2. Extract markdown from first page
        print("\n📝 Extracting Markdown from page 1...")
        markdown = doc.extract_page_markdown(0)

        print("✅ Markdown extraction successful!")
        print("\n--- EXTRACTED MARKDOWN ---")
        print(markdown)
        print("--- END MARKDOWN ---\n")

        # 3. Test with multiple pages if available
        if page_count > 1:
            print(f"📄 Testing page 2 of {page_count}...")
            markdown2 = doc.extract_page_markdown(1)
            print("✅ Page 2 extraction successful!")
            print("\n--- PAGE 2 MARKDOWN (First 200 chars) ---")
            print(markdown2[:200] + "..." if len(markdown2) > 200 else markdown2)
            print("--- END MARKDOWN ---\n")

        print("🎉 Mission 008: ALL TESTS PASSED!")
        return True

    except Exception as e:
        print(f"❌ Test failed: {e}")
        import traceback

        traceback.print_exc()
        return False


if __name__ == "__main__":
    success = test_markdown_extraction()
    sys.exit(0 if success else 1)
