import subprocess
import sys

cmd = ["cargo", "build", "-p", "elite_pdf", "--release", "--verbose"]
print(f"Running: {cmd}")
try:
    res = subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")
    print("--- STDOUT ---")
    print(res.stdout)
    print("--- STDERR ---")
    print(res.stderr)
    
    with open("build_log.txt", "w", encoding="utf-8") as f:
        f.write(res.stdout + "\n" + res.stderr)
except Exception as e:
    print(f"Error: {e}")
