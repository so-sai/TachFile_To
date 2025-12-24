//! IPC Manager for Rust-Python-Frontend communication (Tauri v2)
pub mod protocol;

use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, BufReader, AsyncWriteExt};
use tokio::process::{ChildStdin, ChildStdout};
use tokio::sync::Mutex as AsyncMutex;
use log::{info, warn, error};

pub use protocol::*;

/// Manages Python worker subprocess
pub struct PythonWorker {
    child: Option<Child>,
    stdin: Arc<AsyncMutex<Option<ChildStdin>>>,
    stdout: Arc<AsyncMutex<Option<BufReader<ChildStdout>>>>,
}

impl PythonWorker {
    pub fn new() -> Self {
        Self {
            child: None,
            stdin: Arc::new(AsyncMutex::new(None)),
            stdout: Arc::new(AsyncMutex::new(None)),
        }
    }
    
    /// Spawn Python worker process
    pub async fn spawn(&mut self) -> anyhow::Result<()> {
        info!("Spawning Python worker...");
        
        let mut command = Command::new("python");
        command
            .arg("-m")
            .arg("backend.app.main")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        let mut child = command.spawn()?;
        
        let stdin = child.stdin.take().expect("Failed to get stdin");
        let stdout = child.stdout.take().expect("Failed to get stdout");
        
        let async_stdin = ChildStdin::from_std(stdin)?;
        let async_stdout = ChildStdout::from_std(stdout)?;
        
        *self.stdin.lock().await = Some(async_stdin);
        *self.stdout.lock().await = Some(BufReader::new(async_stdout));
        self.child = Some(child);
        
        info!("Python worker spawned (PID: {:?})", self.child.as_ref().unwrap().id());
        
        Ok(())
    }
    
    /// Send command to Python worker
    pub async fn send_command(&self, cmd: WorkerCommand) -> anyhow::Result<()> {
        let mut stdin_guard = self.stdin.lock().await;
        if let Some(stdin) = stdin_guard.as_mut() {
            let json = serde_json::to_string(&cmd)?;
            stdin.write_all(format!("{}\n", json).as_bytes()).await?;
            info!("Sent command to worker: {}", cmd.id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Python worker not connected"))
        }
    }
    
    /// Read response from Python worker
    pub async fn read_response(&self) -> anyhow::Result<Option<WorkerResponse>> {
        let mut stdout_guard = self.stdout.lock().await;
        if let Some(stdout) = stdout_guard.as_mut() {
            let mut line = String::new();
            let bytes_read = stdout.read_line(&mut line).await?;
            
            if bytes_read == 0 {
                return Ok(None);
            }
            
            let response: WorkerResponse = serde_json::from_str(&line)?;
            Ok(Some(response))
        } else {
            Err(anyhow::anyhow!("Python worker not connected"))
        }
    }
    
    /// Check if worker is alive
    pub fn is_alive(&self) -> bool {
        self.child.as_ref().map_or(false, |_| true)
    }
}

/// Evidence cache manager (Layer 2)
pub struct EvidenceCache {
    memory_cache: Arc<Mutex<lru::LruCache<String, Vec<u8>>>>,
}

impl EvidenceCache {
    pub fn new(max_items: usize) -> Self {
        Self {
            memory_cache: Arc::new(Mutex::new(
                lru::LruCache::new(std::num::NonZeroUsize::new(max_items).unwrap())
            )),
        }
    }
    
    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        let mut cache = self.memory_cache.lock().unwrap();
        cache.get(key).cloned()
    }
    
    pub async fn put(&self, key: String, data: Vec<u8>) {
        let mut cache = self.memory_cache.lock().unwrap();
        cache.put(key, data);
    }
}

// Tauri v2 commands
#[tauri::command]
pub async fn extract_evidence(
    request: EvidenceRequest,
    worker: tauri::State<'_, Arc<AsyncMutex<PythonWorker>>>,
    cache: tauri::State<'_, Arc<EvidenceCache>>,
) -> Result<EvidenceResponse, String> {
    use base64::Engine;
    
    let cache_key = format!("{}:{:?}:{}", request.file_path.display(), request.bbox, request.dpi);
    if let Some(cached) = cache.get(&cache_key).await {
        return Ok(EvidenceResponse::success(
            request.request_id,
            base64::engine::general_purpose::STANDARD.encode(&cached),
            "image/jpeg".to_string(),
            (0, 0),
            true,
        ));
    }
    
    let worker_guard = worker.lock().await;
    let cmd = WorkerCommand {
        id: request.request_id.clone(),
        cmd: "extract_evidence".to_string(),
        payload: serde_json::to_value(&request).map_err(|e| e.to_string())?,
        priority: request.priority,
    };
    
    worker_guard.send_command(cmd).await
        .map_err(|e| format!("Failed to send to worker: {}", e))?;
    
    Ok(EvidenceResponse::pending(request.request_id, 0, 1000))
}

#[tauri::command]
pub async fn get_evidence_health() -> Result<HealthReport, String> {
    Ok(HealthReport {
        status: "healthy".to_string(),
        metrics: HealthMetrics::default(),
        recommendations: vec![],
    })
}

#[tauri::command]
pub async fn clear_evidence_cache(
    cache: tauri::State<'_, Arc<EvidenceCache>>,
) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn restart_evidence_worker(
    worker: tauri::State<'_, Arc<AsyncMutex<PythonWorker>>>,
) -> Result<bool, String> {
    let mut worker_guard = worker.lock().await;
    worker_guard.spawn().await.map_err(|e| e.to_string())?;
    Ok(true)
}
