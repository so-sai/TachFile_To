/*
 * IDLE CONTROLLER - MISSION 012C
 * ==============================
 * 
 * "Người quản gia" (The Janitor's overwatch)
 * Monitors system pressure and idle signals to trigger maintenance.
 */

use std::time::{Duration, Instant};
use crate::executioner::discipline::PressureMonitor;
use crate::executioner::discipline::PressureDecision;
use crate::telemetry_state;

pub struct IdleController {
    pressure_monitor: PressureMonitor,
    idle_threshold: Duration,
    last_activity: Instant,
}

impl IdleController {
    pub fn new(idle_seconds: u64) -> Self {
        Self {
            pressure_monitor: PressureMonitor::new(),
            idle_threshold: Duration::from_secs(idle_seconds),
            last_activity: Instant::now(),
        }
    }

    /// Primary check for maintenance window
    pub fn is_maintenance_window(&mut self) -> bool {
        // 1. Check if we are idle long enough
        if self.last_activity.elapsed() < self.idle_threshold {
            return false;
        }

        // 2. Check system pressure (CPU/RAM)
        match self.pressure_monitor.check() {
            PressureDecision::Normal => {
                // System is quiet and we've been idle
                telemetry_state::set_janitor_status("MAINTENANCE");
                println!("[Mission 012C] System Idle detected. Starting background maintenance...");
                true
            }
            _ => {
                telemetry_state::set_janitor_status("PRESSURE_HIGH");
                false
            },
        }
    }

    /// Reset idle timer (call this on user activity or heavy task start)
    pub fn report_activity(&mut self) {
        telemetry_state::set_janitor_status("BUSY");
        self.last_activity = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_prowler_activation() {
        let mut controller = IdleController::new(1); // 1 second idle
        
        // Immediate check - should be false
        assert!(!controller.is_maintenance_window());
        
        // Wait for idle
        thread::sleep(Duration::from_millis(1100));
        
        // Check again - should be true and print the mission log
        assert!(controller.is_maintenance_window());
    }
}
