import os
import sys
import shutil
import time

# Add the directory containing the compiled .pyd to sys.path
pyd_path = os.path.join(os.getcwd(), "target", "release")
sys.path.append(pyd_path)

try:
    import elite_pdf
    print("Successfully imported elite_pdf")
except ImportError as e:
    print(f"Failed to import elite_pdf: {e}")
    sys.exit(1)

def run_process(filename):
    print(f"\n--- Processing {filename} ---")
    doc = elite_pdf.EliteDocument(filename)
    result = doc.process_output(filename)
    print(f"Result: {result}")
    return result

def test_evidence_generation():
    print("\n--- Testing Evidence Generation ---")
    test_file = "real_test.pdf"
    if not os.path.exists(test_file):
        print(f"Test file {test_file} not found!")
        return

    result = run_process(test_file)
    
    # Extract directory from result string
    # "Prepared multi-page session at: output\20260127_160351_real_test (Splitting pending)"
    if "session at: " in result:
        session_dir = result.split("session at: ")[1].split(" (")[0].strip()
        print(f"Checking session dir: {session_dir}")
        
        # Verify files
        png_exists = os.path.exists(os.path.join(session_dir, "page_1.png"))
        json_exists = os.path.exists(os.path.join(session_dir, "page_1.json"))
        
        print(f"page_1.png exists: {png_exists}")
        print(f"page_1.json exists: {json_exists}")
        
        if json_exists:
            with open(os.path.join(session_dir, "page_1.json"), "r") as f:
                print(f"JSON Content: {f.read()}")

def test_cleanup_rolling_window():
    print("\n--- Testing Cleanup (Rolling Window) ---")
    test_file = "real_test.pdf"
    
    # Run 12 times to exceed the limit of 10
    for i in range(12):
        print(f"Run {i+1}/12...")
        run_process(test_file)
        time.sleep(1.1) # Ensure different timestamps
    
    output_dir = "output"
    sessions = [d for d in os.listdir(output_dir) if os.path.isdir(os.path.join(output_dir, d))]
    print(f"Total sessions in output/: {len(sessions)}")
    print(f"Sessions: {sorted(sessions)}")
    
    if len(sessions) <= 10:
        print("Cleanup working correctly (<= 10 sessions found)")
    else:
        print(f"Cleanup FAILED: {len(sessions)} sessions found (expected <= 10)")

if __name__ == "__main__":
    # Clear output dir first for clean test
    if os.path.exists("output"):
        shutil.rmtree("output")
        
    test_evidence_generation()
    test_cleanup_rolling_window()
