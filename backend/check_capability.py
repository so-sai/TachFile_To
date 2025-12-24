import sys
import os
import mmap

def check_system():
    print(f"🔥 SYSTEM CHECK REPORT", flush=True)
    print(f"----------------------", flush=True)
    print(f"Python Version: {sys.version.split()[0]}", flush=True)
    print(f"Platform: {sys.platform}", flush=True)
    
    # Check 1: Mmap Capability
    try:
        # Tạo file test giả 1MB
        with open("test_mmap.bin", "wb") as f:
            f.write(b"\x00" * 1024 * 1024)
            
        with open("test_mmap.bin", "r+b") as f:
            # Trên Windows, mmap hoạt động khác Linux
            mm = mmap.mmap(f.fileno(), 0, access=mmap.ACCESS_WRITE)
            print(f"✅ MMAP: SUPPORTED (Memory Mapping hoạt động)", flush=True)
            mm.close()
        os.remove("test_mmap.bin")
    except Exception as e:
        print(f"❌ MMAP: FAILED ({e}) - System will fallback to 'Streaming' if necessary.", flush=True)

    # Check 2: Docling Import
    try:
        import docling
        print(f"✅ DOCLING: INSTALLED", flush=True)
    except ImportError:
        print(f"❌ DOCLING: MISSING (Vui lòng chạy 'pip install docling')", flush=True)

if __name__ == "__main__":
    check_system()
