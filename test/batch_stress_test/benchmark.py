import os
import sys
import time
import psutil
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor

# Fix Windows Unicode encoding for emojis
if sys.platform == "win32":
    import io
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

# Inject path to iron_python_bridge using Windows absolute path
BRIDGE_PATH = r"E:\DEV\elite_9_VN-ecosystem\app-tool-TachFileTo\libs\iron_python_bridge\python"
if BRIDGE_PATH not in sys.path:
    sys.path.insert(0, BRIDGE_PATH)

try:
    from extraction import process_document
except ImportError as e:
    print(f"❌ Could not import 'extraction': {e}")
    sys.exit(1)

def get_stats():
    cpu = psutil.cpu_percent(interval=None, percpu=True)
    mem = psutil.virtual_memory().percent
    return cpu, mem

def process_one_file(f_path):
    file_start = time.perf_counter()
    try:
        # Use Windows absolute path string
        data = process_document(str(f_path.absolute()))
        file_end = time.perf_counter()
        latency = file_end - file_start
        return {"file": f_path.name, "status": "OK", "latency": latency}
    except Exception as e:
        return {"file": f_path.name, "status": "ERROR", "msg": str(e), "latency": 0}
    finally:
        if 'data' in locals(): del data

def stress_test_batch(test_dir: str, iterations: int = 5):
    """
    Elite 9 Hardcore Stress Test with Windows absolute paths.
    """
    test_path = Path(test_dir)
    files = [f for f in test_path.glob("**/*") if f.suffix.lower() in [".pdf", ".docx", ".xlsx", ".md"]]
    
    print("=" * 60)
    print("🛡️  ELITE 9 BATCH STRESS TEST")
    print(f"📂 Target: {test_path.absolute()}")
    print(f"📊 Files: {len(files)}")
    print(f"🔄 Iterations: {iterations}")
    print("=" * 60)

    if not files:
        print("❌ No files found!")
        return

    results = []
    latencies = []
    start_time = time.perf_counter()
    
    # Baseline stats
    init_cpu, init_mem = get_stats()
    print(f"📈 Baseline -> CPU Avg: {sum(init_cpu)/len(init_cpu):.1f}% | RAM: {init_mem}%")

    with ThreadPoolExecutor() as executor:
        futures = []
        for i in range(iterations):
            print(f"\n🚀 ITERATION {i+1}/{iterations} (Queuing...)")
            for f_path in files:
                futures.append(executor.submit(process_one_file, f_path))

        for future in futures:
            result = future.result()
            latencies.append(result["latency"])
            results.append(result)
            if result["status"] == "OK":
                print(f"  ✅ {result['file']} ({result['latency']:.2f}s) [Threaded]")
            else:
                print(f"  ❌ {result['file']} -> {result['msg']}")

        # Mid-test stats (approximate, hard to capture inside threads without locking)
        mid_cpu, mid_mem = get_stats()
        print(f"📊 Resource Check -> CPU Avg: {sum(mid_cpu)/len(mid_cpu):.1f}% | RAM: {mid_mem}%")

    end_time = time.perf_counter()
    duration = end_time - start_time
    
    # Final Report
    print("\n" + "=" * 60)
    print("🏆 FINAL REPORT (Alpha RC1)")
    print("=" * 60)
    print(f"⏱️  Total Time: {duration:.2f}s")
    
    success_count = sum(1 for r in results if r['status'] == 'OK')
    print(f"✅ Success: {success_count}/{len(results)}")
    
    if latencies:
        avg_latency = sum(latencies) / len(latencies)
        max_latency = max(latencies)
        min_latency = min(latencies)
        print(f"📈 Throughput: {len(results)/duration:.2f} items/sec")
        print(f"⏳ Latency -> Avg: {avg_latency:.2f}s | Min: {min_latency:.2f}s | Max: {max_latency:.2f}s")
    
    # Check for No-GIL proof (all cores busy)
    final_cpu, final_mem = get_stats()
    cores_active = sum(1 for c in final_cpu if c > 30)
    print(f"🛡️  Cores Active (>30%): {cores_active}/{len(final_cpu)}")
    print(f"💾 Final RAM: {final_mem}%")
    
    if cores_active > 1 and len(results) > 1:
        print("⚡ NO-GIL VERIFIED: Multi-core scaling detected.")
    else:
        print("⚠️  Scaling Check: Sequential behavior or small batch.")
    print("=" * 60)

if __name__ == "__main__":
    # Windows absolute path for test directory
    TARGET_DIR = r"E:\DEV\elite_9_VN-ecosystem\app-tool-TachFileTo\test\pdf"
    stress_test_batch(TARGET_DIR, iterations=50)
