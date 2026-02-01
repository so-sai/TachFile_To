/*
 * PROCESS ISOLATION - MISSION 014
 * ================================
 * 
 * Manages external worker processes for FFI stability.
 */

use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use anyhow::{Result, anyhow};

pub struct WorkerManager {
    child: Arc<Mutex<Option<Child>>>,
    cmd_args: Vec<String>,
}

impl WorkerManager {
    pub fn new(_command: &str, args: &[&str]) -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
            cmd_args: args.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn spawn(&self) -> Result<()> {
        let mut mg = self.child.lock().unwrap();
        
        let child = Command::new("python") 
            .args(&self.cmd_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow!("Failed to spawn extraction worker: {}", e))?;
            
        *mg = Some(child);
        Ok(())
    }

    pub fn is_alive(&self) -> bool {
        let mut mg = self.child.lock().unwrap();
        if let Some(child) = mg.as_mut() {
            match child.try_wait() {
                Ok(None) => true,
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn restart_if_needed(&self) -> Result<()> {
        if !self.is_alive() {
            self.spawn()?;
        }
        Ok(())
    }

    /// Send a JSON command to worker and read response (for persistent workers)
    pub fn send_command(&self, json_payload: &str) -> Result<String> {
        use std::io::{Write, BufRead, BufReader};
        
        let mut mg = self.child.lock().unwrap();
        let child = mg.as_mut().ok_or_else(|| anyhow!("Worker not spawned"))?;
        
        // Write command to stdin
        if let Some(stdin) = child.stdin.as_mut() {
            writeln!(stdin, "{}", json_payload)?;
            stdin.flush()?;
        } else {
            return Err(anyhow!("Worker stdin not available"));
        }
        
        // Read response from stdout
        if let Some(stdout) = child.stdout.as_mut() {
            let mut reader = BufReader::new(stdout);
            let mut response = String::new();
            reader.read_line(&mut response)?;
            Ok(response.trim().to_string())
        } else {
            Err(anyhow!("Worker stdout not available"))
        }
    }

    /// Execute task and wait for completion (legacy method for one-shot workers)
    pub fn execute_task(&self) -> Result<String> {
        let child = {
            let mut mg = self.child.lock().unwrap();
            mg.take().ok_or_else(|| anyhow!("Worker not spawned"))?
        };
        
        let output = child.wait_with_output()
            .map_err(|e| anyhow!("Worker execution failed: {}", e))?;
            
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Worker exited with error: {}", err));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use std::thread;

    #[test]
    fn test_worker_lifecycle() {
        // Use a simple python command that prints "ready" then exits
        let manager = WorkerManager::new("python", &["-c", "print('ready')"]);
        
        // 1. Initial State
        assert!(!manager.is_alive());
        
        // 2. Spawn
        manager.spawn().unwrap();
        // It might be too fast to catch with is_alive if it exits immediately
        // but let's assume it works for 'python -c'
        
        // 3. Execute
        let result = manager.execute_task().unwrap();
        assert!(result.contains("ready"));
        
        // 4. Check Health (should be None after take/execute)
        assert!(!manager.is_alive());
    }
}
