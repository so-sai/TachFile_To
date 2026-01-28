/*
 * RUNTIME DISCIPLINE LAYER - MISSION 012D
 * ========================================
 * 
 * "The Handbrake" (Phanh tay) - Ensures workers are cooperative.
 * 
 * Principles:
 * 1. Pull-based: Workers must actively check checkpoints.
 * 2. Sync-only: No async overhead in the high-frequency loop.
 * 3. Cooperative: Workers yield or abort based on system/court pressure.
 */

use std::time::{Instant, Duration, SystemTime, UNIX_EPOCH};
use crate::executioner::api::QuiesceSignal;

// ============================================================
// 1. DECISION TYPES
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuiesceDecision {
    Proceed,
    YieldUntil(u64), // Unix timestamp (secs)
    Abort,           // Deadline violated or Global Quiesce
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetDecision {
    Continue,
    YieldNow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressureDecision {
    Normal,
    Pause,
    Shed, // Future use: load shedding
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisciplineDecision {
    Continue,
    Yield,
    Abort,
}

// ============================================================
// 2. QUIESCE GATE
// ============================================================

pub struct QuiesceGate;

impl QuiesceGate {
    /// Check if the worker should yield or stop based on the signal
    pub fn check(signal: &QuiesceSignal, file_id_hash: u64) -> QuiesceDecision {
        match signal {
            QuiesceSignal::None => QuiesceDecision::Proceed,
            QuiesceSignal::Pending {
                file_id_hash: target_hash,
                deadline_unix_sec,
            } => {
                if *target_hash == file_id_hash {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    
                    if now >= *deadline_unix_sec {
                        QuiesceDecision::Abort
                    } else {
                        QuiesceDecision::YieldUntil(*deadline_unix_sec)
                    }
                } else {
                    QuiesceDecision::Proceed
                }
            }
            QuiesceSignal::Global { .. } => QuiesceDecision::Abort,
        }
    }
}

// ============================================================
// 3. FRAME BUDGET
// ============================================================

pub struct FrameBudget {
    start: Instant,
    limit: Duration,
}

impl FrameBudget {
    pub fn new(ms_limit: u64) -> Self {
        Self {
            start: Instant::now(),
            limit: Duration::from_millis(ms_limit),
        }
    }

    pub fn default_60fps() -> Self {
        Self::new(16) // ~16ms for 60fps
    }

    pub fn check(&self) -> BudgetDecision {
        if self.start.elapsed() >= self.limit {
            BudgetDecision::YieldNow
        } else {
            BudgetDecision::Continue
        }
    }
}

// ============================================================
// 4. PRESSURE MONITOR
// ============================================================

pub struct PressureMonitor {
    last_check: Instant,
    cached_decision: PressureDecision,
    throttle: Duration,
}

impl PressureMonitor {
    pub fn new() -> Self {
        Self {
            last_check: Instant::now(),
            cached_decision: PressureDecision::Normal,
            throttle: Duration::from_millis(500),
        }
    }

    pub fn check(&mut self) -> PressureDecision {
        if self.last_check.elapsed() >= self.throttle {
            // TODO: In a real implementation, use sysinfo or similar to check RAM usage.
            // For Mission 012D, we provide the structure and placeholder.
            self.cached_decision = self.poll_system();
            self.last_check = Instant::now();
        }
        self.cached_decision
    }

    fn poll_system(&self) -> PressureDecision {
        // Placeholder for real sysinfo call
        PressureDecision::Normal
    }
}

// ============================================================
// 5. DISCIPLINE GUARD (SINGLE POINT OF ENTRY)
// ============================================================

pub struct DisciplineGuard<'a> {
    pub signal: &'a QuiesceSignal,
    pub file_id_hash: u64,
    pub budget: &'a FrameBudget,
    pub pressure: &'a mut PressureMonitor,
}

impl<'a> DisciplineGuard<'a> {
    pub fn checkpoint(&mut self) -> DisciplineDecision {
        // 1. Check Quiesce (Court command) - Highest Priority
        match QuiesceGate::check(self.signal, self.file_id_hash) {
            QuiesceDecision::Abort => return DisciplineDecision::Abort,
            QuiesceDecision::YieldUntil(_) => return DisciplineDecision::Yield,
            QuiesceDecision::Proceed => {}
        }

        // 2. Check Pressure (OS command)
        match self.pressure.check() {
            PressureDecision::Pause | PressureDecision::Shed => return DisciplineDecision::Yield,
            PressureDecision::Normal => {}
        }

        // 3. Check Frame Budget (UI command)
        match self.budget.check() {
            BudgetDecision::YieldNow => return DisciplineDecision::Yield,
            BudgetDecision::Continue => {}
        }

        DisciplineDecision::Continue
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quiesce_gate_respects_deadline() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let file_hash = 12345;
        
        // Signal with future deadline
        let signal = QuiesceSignal::Pending {
            file_id_hash: file_hash,
            deadline_unix_sec: now + 10,
        };
        assert_eq!(QuiesceGate::check(&signal, file_hash), QuiesceDecision::YieldUntil(now + 10));
        
        // Signal with past deadline
        let signal_expired = QuiesceSignal::Pending {
            file_id_hash: file_hash,
            deadline_unix_sec: now - 1,
        };
        assert_eq!(QuiesceGate::check(&signal_expired, file_hash), QuiesceDecision::Abort);

        // Global signal
        let signal_global = QuiesceSignal::Global { deadline_unix_sec: now + 10 };
        assert_eq!(QuiesceGate::check(&signal_global, file_hash), QuiesceDecision::Abort);
    }

    #[test]
    fn test_frame_budget_exhaustion() {
        let budget = FrameBudget::new(10); // 10ms
        assert_eq!(budget.check(), BudgetDecision::Continue);
        
        std::thread::sleep(Duration::from_millis(15));
        assert_eq!(budget.check(), BudgetDecision::YieldNow);
    }

    #[test]
    fn test_discipline_guard_aggregation() {
        let signal = QuiesceSignal::None;
        let budget = FrameBudget::new(100);
        let mut pressure = PressureMonitor::new();
        
        let mut guard = DisciplineGuard {
            signal: &signal,
            file_id_hash: 1,
            budget: &budget,
            pressure: &mut pressure,
        };
        
        assert_eq!(guard.checkpoint(), DisciplineDecision::Continue);

        // Simulate Abort (Global Quiesce)
        let signal_abort = QuiesceSignal::Global { deadline_unix_sec: 9999999999 };
        let mut guard_abort = DisciplineGuard {
            signal: &signal_abort,
            file_id_hash: 1,
            budget: &budget,
            pressure: &mut pressure,
        };
        assert_eq!(guard_abort.checkpoint(), DisciplineDecision::Abort);
    }
}
