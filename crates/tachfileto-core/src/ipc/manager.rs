use crate::ipc::protocol::{IpcMessage, MessageType};
use crate::ipc::router::{MessageRouter, RouterResponse};
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::Mutex;
// use uuid::Uuid; // removed unused import

/// IPC Manager - manages Python worker subprocess and communication
pub struct IpcManager {
    process: Arc<Mutex<Option<Child>>>,
    router: MessageRouter,
    python_path: String,
}

impl IpcManager {
    /// Create a new IPC Manager
    pub fn new(python_path: String) -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
            router: MessageRouter::new(),
            python_path,
        }
    }

    /// Start the Python worker process
    pub async fn start(&self) -> Result<(), String> {
        let mut process_guard = self.process.lock().await;

        if process_guard.is_some() {
            return Err("Worker already running".to_string());
        }

        // Get absolute path to backend
        let mut backend_dir = std::env::current_dir().map_err(|e| e.to_string())?;
        // If we are in crates/tachfileto-core, go up to workspace root
        if backend_dir.ends_with("tachfileto-core") {
            backend_dir.pop();
            backend_dir.pop();
        }
        backend_dir.push("backend");

        eprintln!("📂 Backend directory: {:?}", backend_dir);

        // Spawn Python worker process
        let child = Command::new(&self.python_path)
            .args(&["-m", "app.main"])
            .current_dir(backend_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit()) // Pass stderr through to help debugging
            .spawn()
            .map_err(|e| format!("Failed to spawn Python worker: {}", e))?;

        *process_guard = Some(child);

        eprintln!("✅ Python worker process started");

        Ok(())
    }

    /// Send handshake to Python worker
    pub async fn handshake(&self) -> Result<Value, String> {
        let payload = serde_json::json!({
            "rust_version": "1.83.0",
            "expected_protocol_v": "1.0.0",
            "capabilities_requested": ["ocr", "table_extraction"]
        });

        let msg = IpcMessage::new(MessageType::CmdHandshake, payload);

        self.send_message(&msg).await
    }

    /// Send a message to the Python worker and wait for response
    pub async fn send_message(&self, msg: &IpcMessage<Value>) -> Result<Value, String> {
        let mut process_guard = self.process.lock().await;

        let process = process_guard.as_mut().ok_or("Worker not running")?;

        // Get stdin handle
        let stdin = process.stdin.as_mut().ok_or("Failed to get worker stdin")?;

        // Register request with router
        let rx = self.router.register_request(msg.msg_id);

        // Serialize and send message
        let json_str = msg
            .to_json()
            .map_err(|e| format!("Failed to serialize message: {}", e))?;

        stdin
            .write_all(json_str.as_bytes())
            .map_err(|e| format!("Failed to write to worker stdin: {}", e))?;
        stdin
            .write_all(b"\n")
            .map_err(|e| format!("Failed to write newline: {}", e))?;
        stdin
            .flush()
            .map_err(|e| format!("Failed to flush stdin: {}", e))?;

        eprintln!("📤 Sent to worker: {:?}", msg.msg_type);

        // Read response from stdout
        let stdout = process
            .stdout
            .as_mut()
            .ok_or("Failed to get worker stdout")?;

        let mut reader = BufReader::new(stdout);
        let mut response_line = String::new();

        reader
            .read_line(&mut response_line)
            .map_err(|e| format!("Failed to read worker response: {}", e))?;

        eprintln!("📥 Received from worker: {}", response_line.trim());

        // Dispatch to router (this will resolve the pending request)
        self.router.dispatch(&response_line).await;

        // Wait for router to process response
        match rx.await {
            Ok(RouterResponse::Success(data)) => Ok(data),
            Ok(RouterResponse::Error(err)) => {
                Err(format!("Worker error: {} - {}", err.code, err.message))
            }
            Err(_) => Err("Response channel closed".to_string()),
        }
    }

    /// Spawn a background task to continuously read worker output
    pub async fn start_output_reader(&self) {
        let process = self.process.clone();
        let router = self.router.clone();

        tokio::spawn(async move {
            loop {
                let mut process_guard = process.lock().await;

                if let Some(child) = process_guard.as_mut() {
                    if let Some(stdout) = child.stdout.as_mut() {
                        let mut reader = BufReader::new(stdout);
                        let mut line = String::new();

                        match reader.read_line(&mut line) {
                            Ok(0) => {
                                eprintln!("⚠️ Worker stdout closed");
                                break;
                            }
                            Ok(_) => {
                                eprintln!("📥 Worker output: {}", line.trim());
                                router.dispatch(&line).await;
                            }
                            Err(e) => {
                                eprintln!("❌ Error reading worker output: {}", e);
                                break;
                            }
                        }
                    }
                }

                drop(process_guard);
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        });
    }

    /// Stop the Python worker
    pub async fn stop(&self) -> Result<(), String> {
        let mut process_guard = self.process.lock().await;

        if let Some(mut child) = process_guard.take() {
            child
                .kill()
                .map_err(|e| format!("Failed to kill worker: {}", e))?;

            child
                .wait()
                .map_err(|e| format!("Failed to wait for worker: {}", e))?;

            eprintln!("🛑 Python worker stopped");
        }

        Ok(())
    }
}

impl Drop for IpcManager {
    fn drop(&mut self) {
        // Best effort cleanup
        if let Ok(mut guard) = self.process.try_lock() {
            if let Some(mut child) = guard.take() {
                let _ = child.kill();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Python environment
    async fn test_manager_lifecycle() {
        let manager = IpcManager::new("python".to_string());

        // Start worker
        manager.start().await.expect("Failed to start worker");

        // Wait a bit for initialization
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Send handshake
        let response = manager.handshake().await.expect("Handshake failed");

        assert!(response.get("worker_pid").is_some());
        assert_eq!(
            response.get("status").and_then(|v| v.as_str()),
            Some("ready")
        );

        // Stop worker
        manager.stop().await.expect("Failed to stop worker");
    }
}
