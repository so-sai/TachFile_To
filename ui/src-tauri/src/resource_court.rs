/*
 * MISSION 012A - RESOURCE COURT (TAM QUYỀN PHÂN LẬP)
 * ===================================================
 * Separation of Powers: Registry → Court → Executioner
 * 
 * Phase 1: Policy Engine (Pure Logic, No I/O)
 * 
 * Architecture:
 *   1. CacheRegistry     - Thống kê (Facts only)
 *   2. ResourceCourt     - Tư pháp (Judgment)
 *   3. Executioner       - Hành pháp (Execution)
 *
 * Build Date: 2026-01-28
 * Status: SKELETON - Ready for Phase 2 (Executioner)
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================
// 1. DOMAIN MODELS
// ============================================================

/// File metadata as tracked by registry (FACTS ONLY - no deletion logic here)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub file_id: String,                        // Unique identifier
    pub file_path: String,                      // Relative path
    pub file_size_bytes: u64,                   // Physical size
    pub file_count: usize,                      // Number of files in this entry
    pub created_at: u64,                        // Creation timestamp (UNIX secs)
    pub last_accessed_at: u64,                  // Last access timestamp
    pub access_count: u64,                      // Total access count
    pub user_pinned: bool,                      // User explicitly protected
    pub viewport_distance: f64,                 // Distance from viewport (0.0 = in view)
}

/// Entropy factor - indicates fragmentation risk
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EntropyMetrics {
    pub file_count: usize,                      // Number of files
    pub subdirectory_count: usize,              // Number of subdirs
    pub avg_file_size_bytes: u64,               // Average file size
    pub entropy_factor: f64,                    // Calculated entropy (0.0 to 1.0)
}

/// Eviction Score - deterministic judgment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvictionScore {
    pub file_id: String,
    pub size_component: f64,                    // w1 * size_ratio
    pub age_component: f64,                     // w2 * age_ratio
    pub viewport_component: f64,                // w3 * viewport_distance
    pub entropy_component: f64,                 // w4 * entropy_factor
    pub total_score: f64,                       // Weighted sum (0.0 to 1.0)
    pub severity_level: EvictionSeverity,       // LOW, MEDIUM, HIGH, CRITICAL
}

/// Severity classification for eviction candidates
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvictionSeverity {
    #[serde(rename = "LOW")]
    Low,
    #[serde(rename = "MEDIUM")]
    Medium,
    #[serde(rename = "HIGH")]
    High,
    #[serde(rename = "CRITICAL")]
    Critical,
}

/// Eviction verdict (decision of ResourceCourt)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvictionVerdict {
    pub file_id: String,
    pub action: EvictionAction,                 // What to do
    pub reason: String,                         // Why
    pub score: f64,                             // Score that triggered this
    pub timestamp: u64,                         // When decision was made
    pub is_reversible: bool,                    // Can be undone?
}

/// Possible actions from the court
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EvictionAction {
    #[serde(rename = "RETAIN")]
    Retain,                                     // Keep (protected)
    #[serde(rename = "MONITOR")]
    Monitor,                                    // Watch closely
    #[serde(rename = "SOFT_DELETE")]
    SoftDelete,                                 // Mark for deletion (can recover)
    #[serde(rename = "HARD_DELETE")]
    HardDelete,                                 // Irreversible deletion
}

/// Policy configuration (Immutable Constitution)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvictionPolicy {
    pub max_cache_size_bytes: u64,              // Total cache size limit
    pub min_age_seconds: u64,                   // Minimum age before eligible
    pub size_weight: f64,                       // w1 (default: 0.25)
    pub age_weight: f64,                        // w2 (default: 0.25)
    pub viewport_weight: f64,                   // w3 (default: 0.30)
    pub entropy_weight: f64,                    // w4 (default: 0.20)
    
    // Entropy thresholds
    pub entropy_high_file_count: usize,         // Files per dir threshold (default: 10000)
    pub entropy_warning_threshold: f64,         // Entropy score > this triggers action (default: 0.6)
    
    // Severity mapping
    pub eviction_threshold_critical: f64,       // Score > this = CRITICAL (default: 0.8)
    pub eviction_threshold_high: f64,           // Score > this = HIGH (default: 0.6)
    pub eviction_threshold_medium: f64,         // Score > this = MEDIUM (default: 0.4)
    
    // Hard limits
    pub max_files_per_directory: usize,         // Sanity check (default: 50000)
    pub purge_all_enabled: bool,                // Is manual "purge all" allowed? (default: false)
}

// ============================================================
// 2. CACHE REGISTRY (TAM THỊ QUAN SÁT)
// ============================================================

/// The Registry - keeps facts, makes NO decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheRegistry {
    entries: HashMap<String, CacheEntry>,
    total_size_bytes: u64,
    last_updated: u64,
}

impl CacheRegistry {
    /// Create new empty registry
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            total_size_bytes: 0,
            last_updated: current_timestamp(),
        }
    }

    /// Register a file (observation only, no judgment)
    pub fn register_entry(&mut self, entry: CacheEntry) {
        self.total_size_bytes += entry.file_size_bytes;
        self.entries.insert(entry.file_id.clone(), entry);
        self.last_updated = current_timestamp();
    }

    /// Update access time for a file
    pub fn touch_entry(&mut self, file_id: &str) -> bool {
        if let Some(entry) = self.entries.get_mut(file_id) {
            entry.last_accessed_at = current_timestamp();
            entry.access_count += 1;
            self.last_updated = current_timestamp();
            true
        } else {
            false
        }
    }

    /// Get all entries (for iteration by Court)
    pub fn entries(&self) -> &HashMap<String, CacheEntry> {
        &self.entries
    }

    /// Get total registered cache size
    pub fn total_size_bytes(&self) -> u64 {
        self.total_size_bytes
    }

    /// Statistics (informational only)
    pub fn stats(&self) -> RegistryStats {
        RegistryStats {
            entry_count: self.entries.len(),
            total_size_bytes: self.total_size_bytes,
            last_updated: self.last_updated,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    pub entry_count: usize,
    pub total_size_bytes: u64,
    pub last_updated: u64,
}

impl Default for CacheRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 3. RESOURCE COURT (TOÀ ÁN TÀI NGUYÊN)
// ============================================================

/// The Judge - applies policy and renders verdict
pub struct ResourceCourt {
    policy: EvictionPolicy,
    judgment_log: Vec<EvictionVerdict>,
}

impl ResourceCourt {
    /// Create new court with policy
    pub fn new(policy: EvictionPolicy) -> Self {
        Self {
            policy,
            judgment_log: Vec::new(),
        }
    }

    /// Calculate eviction score for a single entry
    pub fn calculate_eviction_score(
        &self,
        entry: &CacheEntry,
        entropy: &EntropyMetrics,
        current_time: u64,
    ) -> EvictionScore {
        // 1. Size Component (w1)
        let size_ratio = (entry.file_size_bytes as f64) / (self.policy.max_cache_size_bytes as f64);
        let size_ratio_clamped = size_ratio.min(1.0);
        let size_component = self.policy.size_weight * size_ratio_clamped;

        // 2. Age Component (w2)
        let age_seconds = current_time.saturating_sub(entry.created_at);
        let age_factor = if age_seconds > self.policy.min_age_seconds {
            let normalized_age = (age_seconds as f64) / (86400.0 * 30.0); // 30 days
            normalized_age.min(1.0)
        } else {
            0.0
        };
        let age_component = self.policy.age_weight * age_factor;

        // 3. Viewport Distance Component (w3)
        let viewport_component = self.policy.viewport_weight * entry.viewport_distance.min(1.0);

        // 4. Entropy Component (w4)
        let entropy_component = self.policy.entropy_weight * entropy.entropy_factor;

        // Total weighted score
        let total_score = size_component
            + age_component
            + viewport_component
            + entropy_component;

        // Determine severity
        let severity_level = match total_score {
            s if s >= self.policy.eviction_threshold_critical => EvictionSeverity::Critical,
            s if s >= self.policy.eviction_threshold_high => EvictionSeverity::High,
            s if s >= self.policy.eviction_threshold_medium => EvictionSeverity::Medium,
            _ => EvictionSeverity::Low,
        };

        EvictionScore {
            file_id: entry.file_id.clone(),
            size_component,
            age_component,
            viewport_component,
            entropy_component,
            total_score,
            severity_level,
        }
    }

    /// Judge all entries and return verdicts
    pub fn judge_entries(
        &mut self,
        registry: &CacheRegistry,
        current_cache_size: u64,
    ) -> Vec<EvictionVerdict> {
        let mut verdicts = Vec::new();
        let current_time = current_timestamp();

        for entry in registry.entries().values() {
            // Calculate entropy for this entry
            let entropy = calculate_entropy(&entry);

            // Calculate eviction score
            let score = self.calculate_eviction_score(entry, &entropy, current_time);

            // Render verdict based on score and policy
            let verdict = self.render_verdict(entry, score, current_cache_size);

            verdicts.push(verdict.clone());
            self.judgment_log.push(verdict);
        }

        verdicts
    }

    /// Render a verdict based on score
    fn render_verdict(
        &self,
        entry: &CacheEntry,
        score: EvictionScore,
        current_cache_size: u64,
    ) -> EvictionVerdict {
        let action = if entry.user_pinned {
            // User protection is absolute
            EvictionAction::Retain
        } else if entry.viewport_distance < 0.1 && entry.access_count > 5 {
            // Item is in viewport and accessed frequently -> retain
            EvictionAction::Retain
        } else if score.severity_level == EvictionSeverity::Critical && current_cache_size > self.policy.max_cache_size_bytes {
            // Cache is over limit and this item is critical -> hard delete
            EvictionAction::HardDelete
        } else if score.severity_level >= EvictionSeverity::High {
            // High severity -> soft delete (reversible)
            EvictionAction::SoftDelete
        } else if score.severity_level == EvictionSeverity::Medium {
            // Medium severity -> monitor closely
            EvictionAction::Monitor
        } else {
            // Low severity -> retain for now
            EvictionAction::Retain
        };

        let reason = format!(
            "Size: {:.2}, Age: {:.2}, Viewport: {:.2}, Entropy: {:.2} (Total: {:.2})",
            score.size_component,
            score.age_component,
            score.viewport_component,
            score.entropy_component,
            score.total_score
        );

        EvictionVerdict {
            file_id: entry.file_id.clone(),
            action,
            reason,
            score: score.total_score,
            timestamp: current_timestamp(),
            is_reversible: action != EvictionAction::HardDelete,
        }
    }

    /// Get judgment log (audit trail)
    pub fn judgment_log(&self) -> &[EvictionVerdict] {
        &self.judgment_log
    }

    /// Get current policy
    pub fn policy(&self) -> &EvictionPolicy {
        &self.policy
    }
}

// ============================================================
// 4. DEFAULT POLICIES
// ============================================================

impl Default for EvictionPolicy {
    fn default() -> Self {
        Self {
            max_cache_size_bytes: 500 * 1024 * 1024,              // 500 MB
            min_age_seconds: 86400,                               // 1 day
            size_weight: 0.25,
            age_weight: 0.25,
            viewport_weight: 0.30,
            entropy_weight: 0.20,
            entropy_high_file_count: 10000,
            entropy_warning_threshold: 0.6,
            eviction_threshold_critical: 0.8,
            eviction_threshold_high: 0.6,
            eviction_threshold_medium: 0.4,
            max_files_per_directory: 50000,
            purge_all_enabled: false,
        }
    }
}

// ============================================================
// 5. HELPER FUNCTIONS
// ============================================================

/// Get current UNIX timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Calculate entropy metrics for an entry
fn calculate_entropy(entry: &CacheEntry) -> EntropyMetrics {
    // Heuristic: Entropy factor = (file_count / threshold)
    // Files with very small size but high count = high entropy (bad for filesystem)
    
    let entropy_factor = if entry.file_count == 0 {
        0.0
    } else {
        // Ratio of files to some expected threshold
        let ratio = (entry.file_count as f64) / 1000.0;  // Reference: 1000 files
        ratio.min(1.0)  // Clamp to [0, 1]
    };

    EntropyMetrics {
        file_count: entry.file_count,
        subdirectory_count: 1,  // Simplified - in real implementation, scan filesystem
        avg_file_size_bytes: if entry.file_count > 0 {
            entry.file_size_bytes / entry.file_count as u64
        } else {
            0
        },
        entropy_factor,
    }
}

// ============================================================
// 6. TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entry(file_id: &str, size: u64, age_days: u64) -> CacheEntry {
        let now = current_timestamp();
        let created_at = now - (age_days * 86400);

        CacheEntry {
            file_id: file_id.to_string(),
            file_path: format!("/cache/{}", file_id),
            file_size_bytes: size,
            file_count: 1,
            created_at,
            last_accessed_at: now,
            access_count: 5,
            user_pinned: false,
            viewport_distance: 0.5,
        }
    }

    #[test]
    fn test_registry_basic_operations() {
        let mut registry = CacheRegistry::new();
        let entry = create_test_entry("file1", 1024 * 1024, 5);  // 1 MB, 5 days old

        registry.register_entry(entry.clone());

        assert_eq!(registry.total_size_bytes(), 1024 * 1024);
        assert_eq!(registry.entries().len(), 1);
        assert!(registry.touch_entry("file1"));
        assert!(!registry.touch_entry("nonexistent"));
    }

    #[test]
    fn test_court_eviction_score_calculation() {
        let policy = EvictionPolicy::default();
        let court = ResourceCourt::new(policy);
        let entry = create_test_entry("file1", 100 * 1024 * 1024, 30);  // 100 MB, 30 days

        let entropy = calculate_entropy(&entry);
        let score = court.calculate_eviction_score(&entry, &entropy, current_timestamp());

        println!(
            "Score: total={:.3}, size={:.3}, age={:.3}, viewport={:.3}, entropy={:.3}",
            score.total_score,
            score.size_component,
            score.age_component,
            score.viewport_component,
            score.entropy_component
        );

        assert!(score.total_score >= 0.0 && score.total_score <= 1.0);
        assert!(score.size_component >= 0.0 && score.size_component <= 0.25);
    }

    #[test]
    fn test_court_judgment_with_pinned_entry() {
        let policy = EvictionPolicy::default();
        let mut court = ResourceCourt::new(policy);
        let mut registry = CacheRegistry::new();

        let mut entry = create_test_entry("file1", 100 * 1024 * 1024, 30);
        entry.user_pinned = true;  // User protection

        registry.register_entry(entry);

        let verdicts = court.judge_entries(&registry, 400 * 1024 * 1024);

        assert_eq!(verdicts.len(), 1);
        assert_eq!(verdicts[0].action, EvictionAction::Retain);
    }

    #[test]
    fn test_entropy_calculation() {
        let entry = CacheEntry {
            file_id: "test".to_string(),
            file_path: "/cache/test".to_string(),
            file_size_bytes: 1024 * 1024,
            file_count: 50000,  // High file count = high entropy
            created_at: current_timestamp(),
            last_accessed_at: current_timestamp(),
            access_count: 10,
            user_pinned: false,
            viewport_distance: 0.0,
        };

        let entropy = calculate_entropy(&entry);
        assert!(entropy.entropy_factor > 0.0);
        println!("Entropy factor for 50000 files: {:.3}", entropy.entropy_factor);
    }

    #[test]
    fn test_multiple_entries_judgment() {
        let policy = EvictionPolicy::default();
        let mut court = ResourceCourt::new(policy);
        let mut registry = CacheRegistry::new();

        // Create entries with different characteristics
        registry.register_entry(create_test_entry("old_large", 200 * 1024 * 1024, 60));
        registry.register_entry(create_test_entry("young_small", 10 * 1024 * 1024, 1));
        registry.register_entry(create_test_entry("medium", 100 * 1024 * 1024, 15));

        let verdicts = court.judge_entries(&registry, 350 * 1024 * 1024);

        println!("Verdicts:");
        for v in &verdicts {
            println!("  {} -> {} (score: {:.3})", v.file_id, format!("{:?}", v.action), v.score);
        }

        assert_eq!(verdicts.len(), 3);
    }
}
