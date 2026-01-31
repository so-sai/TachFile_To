import sys
import os

# Add release dir to path
release_dir = os.path.join(os.getcwd(), "target", "release")
sys.path.append(release_dir)

print(f"Searching for module in: {release_dir}")

try:
    import elite_pdf
    print("SUCCESS: Imported elite_pdf")
    
    # Test Stubbed Logic
    try:
        doc = elite_pdf.EliteDocument("dummy.pdf")
        print("SUCCESS: Instantiated EliteDocument")
        
        pages = doc.count_pages()
        print(f"Page Count: {pages}")
        
        if pages == 112:
            print("VERIFICATION PASSED: Stubbed value 112 returned.")
        else:
            print(f"VERIFICATION FAILED: Expected 112, got {pages}")
            
    except Exception as e:
        print(f"FAILURE during usage: {e}")

except ImportError as e:
    print(f"FAILURE to import: {e}")
    # Check if file exists
    pyd_path = os.path.join(release_dir, "elite_pdf.pyd")
    if os.path.exists(pyd_path):
        print(f"File exists at {pyd_path}")
    else:
        print(f"File NOT found at {pyd_path}")
