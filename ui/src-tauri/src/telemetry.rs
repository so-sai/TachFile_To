use serde::Serialize;
use tauri::{AppHandle, Emitter};
use std::time::Duration;
use tokio::time::sleep;
// use sysinfo::{System}; // User prefers specific system monitoring
use sysinfo::{System, Pid};
use crate::telemetry_state;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryPayload {
    pub cpu_usage: f32,
    pub ram_usage_mb: u64,
    pub ffi_lock_active: bool,
    pub is_limp_mode: bool,
    pub janitor_status: String,
}

struct SystemMonitor {
    sys: System,
    pid: Pid,
}

impl SystemMonitor {
    fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let pid = Pid::from_u32(std::process::id());
        Self { sys, pid }
    }

    fn update(&mut self) -> (f32, u64) {
        self.sys.refresh_all();
        
        let global_cpu = self.sys.global_cpu_usage();
        
        let ram_mb = if let Some(process) = self.sys.process(self.pid) {
            process.memory() / 1024 / 1024
        } else {
            0
        };

        (global_cpu, ram_mb)
    }
}

pub async fn start_telemetry_loop(app_handle: AppHandle) {
    let monitor = Arc::new(Mutex::new(SystemMonitor::new()));
    
    loop {
        let (cpu, ram) = {
            let mut m = monitor.lock().unwrap();
            m.update()
        };

        // Fetch from global state
        let ffi_lock_active = telemetry_state::FFI_LOCK_ACTIVE.load(Ordering::Relaxed);
        let is_limp_mode = telemetry_state::IS_LIMP_MODE.load(Ordering::Relaxed);
        let janitor_status = if let Ok(s) = telemetry_state::JANITOR_STATUS.lock() {
            s.clone()
        } else {
            "IDLE".to_string()
        };

        let payload = TelemetryPayload {
            cpu_usage: cpu,
            ram_usage_mb: ram,
            ffi_lock_active,
            is_limp_mode,
            janitor_status,
        };

        let _ = app_handle.emit("system:heartbeat", &payload);
        
        sleep(Duration::from_millis(1000)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_payload_serialization() {
        let payload = TelemetryPayload {
            cpu_usage: 45.5,
            ram_usage_mb: 1024,
            ffi_lock_active: true,
            is_limp_mode: false,
            janitor_status: "BUSY".to_string(),
        };

        let json = serde_json::to_string(&payload).expect("Failed to serialize");
        
        // Assertions for Frontend Contract (camelCase as per #[serde(rename_all = "camelCase")])
        assert!(json.contains("\"cpuUsage\":45.5"));
        assert!(json.contains("\"ramUsageMb\":1024"));
        assert!(json.contains("\"ffiLockActive\":true"));
        assert!(json.contains("\"janitorStatus\":\"BUSY\""));
        
        println!("✅ Telemetry Contract Verified: {}", json);
    }

    #[test]
    fn test_ffi_lock_event_integrity() {
        use crate::telemetry_state;
        
        // 1. Set state to locked
        telemetry_state::set_ffi_lock(true);
        telemetry_state::set_janitor_status("MAINTENANCE");
        
        // 2. Mock loop logic (without AppHandle)
        let ffi_lock_active = telemetry_state::FFI_LOCK_ACTIVE.load(Ordering::Relaxed);
        let janitor_status = if let Ok(s) = telemetry_state::JANITOR_STATUS.lock() {
            s.clone()
        } else {
            "IDLE".to_string()
        };

        assert!(ffi_lock_active);
        assert_eq!(janitor_status, "MAINTENANCE");

        // 3. Reset state
        telemetry_state::set_ffi_lock(false);
        assert!(!telemetry_state::FFI_LOCK_ACTIVE.load(Ordering::Relaxed));
        
        println!("✅ FFI Lock Event Integrity Verified (Logic Layer)");
    }
}
