import os
import sys
import shutil
from datetime import datetime

# Add the directory containing the compiled .pyd to sys.path
# Assuming it builds in target/release
pyd_path = os.path.join(os.getcwd(), "target", "release")
sys.path.append(pyd_path)

try:
    import elite_pdf
    print("Successfully imported elite_pdf")
except ImportError as e:
    print(f"Failed to import elite_pdf: {e}")
    # Attempt to find the .pyd and rename if necessary (sometimes it's elite_pdf.dll)
    # But usually maturin or cargo-cp-pyd handles this. 
    # Since we used cargo build, we might need to copy/rename.
    sys.exit(1)

def test_single_page():
    print("\n--- Testing Single Page PDF ---")
    test_file = "real_test.pdf" # This looks like a real PDF
    if not os.path.exists(test_file):
        print(f"Test file {test_file} not found!")
        return

    doc = elite_pdf.EliteDocument(test_file)
    count = doc.count_pages()
    print(f"Page count: {count}")
    
    result = doc.process_output(test_file)
    print(f"Result: {result}")
    
    # Verify directory creation
    output_dir = "output"
    if os.path.exists(output_dir):
        subdirs = os.listdir(output_dir)
        print(f"Subdirectories: {subdirs}")
        # Find the latest one
        latest_dir = sorted(subdirs)[-1]
        print(f"Latest output dir: {latest_dir}")
        full_path = os.path.join(output_dir, latest_dir)
        print(f"Contents of {latest_dir}: {os.listdir(full_path)}")
    else:
        print("Error: output directory not created!")

def test_multi_page():
    print("\n--- Testing Multi Page PDF ---")
    test_file = "real_test.pdf" # Assuming this has > 1 page
    if not os.path.exists(test_file):
        print(f"Test file {test_file} not found!")
        return

    doc = elite_pdf.EliteDocument(test_file)
    count = doc.count_pages()
    print(f"Page count: {count}")
    
    result = doc.process_output(test_file)
    print(f"Result: {result}")

if __name__ == "__main__":
    test_single_page()
    test_multi_page()
