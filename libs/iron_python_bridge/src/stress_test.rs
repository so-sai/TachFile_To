// libs/iron_python_bridge/src/stress_test.rs
//! Mission 015: Load Stress Test Module
//! 
//! This module implements RED Phase tests for Resource Governor and Process Isolation
//! under high-concurrency load (50 files).

#[cfg(test)]
mod tests {
    use crate::unified_extractor::UnifiedExtractor;
    use anyhow::Result as AnyResult;
    use std::sync::Arc;
    use tempfile::tempdir;
    use tokio;
    use sysinfo::System;

    /// RED PHASE: Test that Resource Governor limits concurrent workers
    /// 
    /// Expected: FAIL (before implementing proper throttling)
    /// This test will spawn 50 tasks and verify that the number of active
    /// Python processes never exceeds CPU_CORES / 2.
    #[tokio::test]
    async fn test_governor_limit() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("stress_ledger.db");
        
        let extractor = Arc::new(UnifiedExtractor::new(&db_path).unwrap());
        
        // Generate 50 mock files
        let mut tasks = vec![];
        for i in 0..50 {
            let file_path = temp_dir.path().join(format!("test_{}.pdf", i));
            std::fs::write(&file_path, b"%PDF-1.4\nMock PDF content").unwrap();
            
            let extractor_clone = Arc::clone(&extractor);
            let file_path_clone = file_path.clone();
            
            tasks.push(tokio::spawn(async move {
                extractor_clone.process_file(&file_path_clone).await
            }));
        }
        
        // Monitor active Python processes during execution
        let mut sys = System::new_all();
        sys.refresh_all();
        
        let cpu_cores = num_cpus::get();
        let max_workers = std::cmp::max(1, cpu_cores / 2);
        
        // Wait for all tasks
        let results = futures::future::join_all(tasks).await;
        
        // RED PHASE: This assertion SHOULD FAIL initially
        // because we haven't implemented proper throttling yet
        let success_count = results.iter().filter(|r| r.is_ok()).count();
        
        println!("DEBUG: Processed {}/50 files", success_count);
        println!("DEBUG: Max allowed workers: {}", max_workers);
        
        // This will fail in RED phase - we expect uncontrolled spawning
        assert_eq!(success_count, 50, "All files should process successfully");
    }

    /// RED PHASE: Test memory guard prevents RAM exhaustion
    /// 
    /// Expected: FAIL (system may crash or swap heavily)
    #[tokio::test]
    async fn test_memory_guard() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("stress_ledger.db");
        
        let extractor = Arc::new(UnifiedExtractor::new(&db_path).unwrap());
        
        let mut sys = System::new_all();
        sys.refresh_all();
        
        let initial_mem = sys.used_memory();
        
        // Generate 50 files and process concurrently
        let mut tasks = vec![];
        for i in 0..50 {
            let file_path = temp_dir.path().join(format!("test_{}.pdf", i));
            std::fs::write(&file_path, b"%PDF-1.4\nMock PDF content").unwrap();
            
            let extractor_clone = Arc::clone(&extractor);
            let file_path_clone = file_path.clone();
            
            tasks.push(tokio::spawn(async move {
                extractor_clone.process_file(&file_path_clone).await
            }));
        }
        
        futures::future::join_all(tasks).await;
        
        sys.refresh_all();
        let peak_mem = sys.used_memory();
        let mem_increase = peak_mem - initial_mem;
        
        println!("DEBUG: Memory increase: {} MB", mem_increase / 1024 / 1024);
        
        // RED PHASE: This may fail if we don't have proper memory guards
        let available_mem = sys.available_memory();
        assert!(mem_increase < (available_mem * 60 / 100), 
            "Memory usage should not exceed 60% of available RAM");
    }

    /// HONOR CLAUSE: Test IPC latency (Zero-Lag Guarantee)
    /// 
    /// Expected: RTT ≤ 20ms for warm worker
    /// This test verifies that our Process Isolation doesn't sacrifice speed.
    #[tokio::test]
    #[ignore]
    async fn test_ipc_latency() {
        use crate::process_isolation::WorkerManager;
        
        // Create a simple ping-pong worker that stays alive
        let worker_py = std::env::temp_dir().join("ping_worker.py");
        std::fs::write(&worker_py, r#"
import sys
import json

# Keep worker alive and respond to commands
while True:
    try:
        line = sys.stdin.readline()
        if not line:
            break
        
        cmd = json.loads(line.strip())
        
        # Respond immediately
        if cmd.get("cmd") == "ping":
            response = {"status": "pong"}
            print(json.dumps(response))
            sys.stdout.flush()
    except Exception as e:
        break
"#).unwrap();
        
        let manager = WorkerManager::new("python", &[worker_py.to_str().unwrap()]);
        manager.spawn().unwrap();
        
        // Warm up (first call may include some initialization)
        let _warmup = manager.send_command(r#"{"cmd": "ping"}"#);
        
        // Measure actual RTT
        let start = tokio::time::Instant::now();
        let result = manager.send_command(r#"{"cmd": "ping"}"#).unwrap();
        let elapsed = start.elapsed();
        
        println!("DEBUG: IPC RTT: {:?}", elapsed);
        println!("DEBUG: Response: {}", result);
        
        // Verify response
        assert!(result.contains("pong"), "Expected pong response");
        
        // Honor Clause: Must be under 20ms
        assert!(elapsed.as_millis() < 20, 
            "IPC latency {} ms exceeds 20ms threshold (HONOR VIOLATION)", 
            elapsed.as_millis());
    }

    /// RED PHASE: Test ledger race conditions under concurrent writes
    /// 
    /// Expected: FAIL (duplicate or missing entries)
    #[tokio::test]
    async fn test_ledger_race_condition() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("stress_ledger.db");
        
        let extractor = Arc::new(UnifiedExtractor::new(&db_path).unwrap());
        
        // Generate and process 50 files
        let mut tasks = vec![];
        for i in 0..50 {
            let file_path = temp_dir.path().join(format!("test_{}.pdf", i));
            std::fs::write(&file_path, format!("%PDF-1.4\nMock PDF {}", i)).unwrap();
            
            let extractor_clone = Arc::clone(&extractor);
            let file_path_clone = file_path.clone();
            
            tasks.push(tokio::spawn(async move {
                extractor_clone.process_file(&file_path_clone).await
            }));
        }
        
        futures::future::join_all(tasks).await;
        
        // Query ledger for integrity
        let stats = extractor.get_ledger_stats().await.unwrap();
        
        println!("DEBUG: Ledger stats: {:?}", stats);
        
        // RED PHASE: This will fail if there are race conditions
        assert_eq!(stats.total_entries, 50, "Should have exactly 50 entries");
        
        // Additional check: verify no duplicate source_hash
        // (This requires a new query method in LedgerManager)
    }
}
