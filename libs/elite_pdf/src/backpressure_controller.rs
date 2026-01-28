use crate::cache_registry::CacheRegistry;
use crate::prefetch_engine::{IntentAwarePrefetcher, PrefetchType};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct WorkItem {
    pub page_id: u32,
    pub work_type: WorkType,
    pub priority: f64,
    pub created_at: Instant,
    pub backpressure_sensitive: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkType {
    SemanticExtraction,
    ImageRendering,
    Both,
}

#[derive(Debug)]
pub struct BackpressureController {
    cache: Arc<CacheRegistry>,
    prefetcher: Arc<IntentAwarePrefetcher>,

    // Work queue management
    work_queue: Arc<Mutex<VecDeque<WorkItem>>>,
    active_workers: Arc<Mutex<usize>>,

    // Backpressure thresholds
    max_concurrent_workers: usize,
    memory_pressure_threshold: f64, // 0.0 - 1.0
    queue_pressure_threshold: usize,

    // Adaptive control
    current_worker_limit: Arc<Mutex<usize>>,
    last_adjustment: Arc<Mutex<Instant>>,

    // Metrics
    total_processed: Arc<Mutex<usize>>,
    rejected_due_to_pressure: Arc<Mutex<usize>>,
}

impl BackpressureController {
    pub fn new(cache: Arc<CacheRegistry>, prefetcher: Arc<IntentAwarePrefetcher>) -> Self {
        Self {
            cache: cache.clone(),
            prefetcher: prefetcher.clone(),
            work_queue: Arc::new(Mutex::new(VecDeque::new())),
            active_workers: Arc::new(Mutex::new(0)),
            max_concurrent_workers: 8,       // Conservative default
            memory_pressure_threshold: 0.85, // 85% memory usage
            queue_pressure_threshold: 20,    // Max queue size
            current_worker_limit: Arc::new(Mutex::new(4)), // Start conservative
            last_adjustment: Arc::new(Mutex::new(Instant::now())),
            total_processed: Arc::new(Mutex::new(0)),
            rejected_due_to_pressure: Arc::new(Mutex::new(0)),
        }
    }

    pub fn submit_work(&self, work_item: WorkItem) -> Result<(), String> {
        // Check backpressure conditions
        if self.should_reject_work(&work_item) {
            *self.rejected_due_to_pressure.lock().unwrap() += 1;
            return Err("Work rejected due to backpressure".to_string());
        }

        // Add to queue
        {
            let mut queue = self.work_queue.lock().unwrap();
            queue.push_back(work_item);

            // Sort by priority (highest first)
            queue
                .make_contiguous()
                .sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());
        }

        // Trigger worker if needed
        self.maybe_spawn_worker();

        Ok(())
    }

    fn should_reject_work(&self, work_item: &WorkItem) -> bool {
        // 1. Check memory pressure
        let (semantic_usage, image_usage) = self.cache.get_memory_stats();
        let total_memory = 100 * 1024 * 1024 + 500 * 1024 * 1024; // 600MB total
        let memory_pressure = (semantic_usage + image_usage) as f64 / total_memory as f64;

        if memory_pressure > self.memory_pressure_threshold {
            return true;
        }

        // 2. Check queue pressure
        let queue_size = self.work_queue.lock().unwrap().len();
        if queue_size > self.queue_pressure_threshold {
            return true;
        }

        // 3. Check specific cache pressure for work type
        match work_item.work_type {
            WorkType::SemanticExtraction | WorkType::Both => {
                if !self.cache.can_accept_semantic_work() {
                    return true;
                }
            }
            WorkType::ImageRendering | WorkType::Both => {
                if !self.cache.can_accept_image_work() {
                    return true;
                }
            }
        }

        // 4. Check worker pressure
        let active = *self.active_workers.lock().unwrap();
        let limit = *self.current_worker_limit.lock().unwrap();
        if active >= limit {
            return true;
        }

        false
    }

    fn maybe_spawn_worker(&self) {
        let active = *self.active_workers.lock().unwrap();
        let limit = *self.current_worker_limit.lock().unwrap();

        if active < limit {
            let queue = self.work_queue.clone();
            let active_workers = self.active_workers.clone();
            let cache = self.cache.clone();
            let total_processed = self.total_processed.clone();

            thread::spawn(move || {
                *active_workers.lock().unwrap() += 1;

                loop {
                    let work_item = {
                        let mut q = queue.lock().unwrap();
                        q.pop_front()
                    };

                    match work_item {
                        Some(item) => {
                            // Process the work item
                            if let Err(e) = Self::process_work_item(&item, &cache) {
                                eprintln!("Worker error processing page {}: {}", item.page_id, e);
                            }

                            *total_processed.lock().unwrap() += 1;
                        }
                        None => {
                            // No more work, exit
                            break;
                        }
                    }

                    // Brief pause to prevent CPU spinning
                    thread::sleep(Duration::from_millis(10));
                }

                *active_workers.lock().unwrap() -= 1;
            });
        }
    }

    fn process_work_item(work_item: &WorkItem, cache: &CacheRegistry) -> Result<(), String> {
        match work_item.work_type {
            WorkType::SemanticExtraction => {
                // Simulate semantic extraction work
                thread::sleep(Duration::from_millis(50));
                // In real implementation, this would call the actual extraction logic
                Ok(())
            }
            WorkType::ImageRendering => {
                // Simulate image rendering work
                thread::sleep(Duration::from_millis(100));
                // In real implementation, this would call the actual rendering logic
                Ok(())
            }
            WorkType::Both => {
                // Process both sequentially
                Self::process_work_item(
                    &WorkItem {
                        page_id: work_item.page_id,
                        work_type: WorkType::SemanticExtraction,
                        priority: work_item.priority,
                        created_at: work_item.created_at,
                        backpressure_sensitive: work_item.backpressure_sensitive,
                    },
                    cache,
                )?;

                Self::process_work_item(
                    &WorkItem {
                        page_id: work_item.page_id,
                        work_type: WorkType::ImageRendering,
                        priority: work_item.priority,
                        created_at: work_item.created_at,
                        backpressure_sensitive: work_item.backpressure_sensitive,
                    },
                    cache,
                )?;

                Ok(())
            }
        }
    }

    pub fn adjust_worker_limits(&self) {
        let now = Instant::now();
        let mut last_adj = self.last_adjustment.lock().unwrap();

        // Only adjust every 5 seconds
        if now.duration_since(*last_adj) < Duration::from_secs(5) {
            return;
        }

        *last_adj = now;

        let (semantic_usage, image_usage) = self.cache.get_memory_stats();
        let total_memory = 100 * 1024 * 1024 + 500 * 1024 * 1024; // 600MB total
        let memory_pressure = (semantic_usage + image_usage) as f64 / total_memory as f64;

        let queue_size = self.work_queue.lock().unwrap().len();
        let active = *self.active_workers.lock().unwrap();

        let mut limit = self.current_worker_limit.lock().unwrap();

        // Adaptive adjustment logic
        if memory_pressure > 0.9 || queue_size > self.queue_pressure_threshold {
            // High pressure - reduce workers
            *limit = (*limit / 2).max(1);
        } else if memory_pressure < 0.6 && queue_size < 5 && active < *limit {
            // Low pressure - can increase workers
            *limit = (*limit + 1).min(self.max_concurrent_workers);
        }

        println!(
            "Backpressure adjustment: memory={:.2}, queue={}, workers={}/{}",
            memory_pressure, queue_size, active, *limit
        );
    }

    pub fn get_backpressure_stats(&self) -> BackpressureStats {
        let (semantic_usage, image_usage) = self.cache.get_memory_stats();
        let total_memory = 100 * 1024 * 1024 + 500 * 1024 * 1024;
        let memory_pressure = (semantic_usage + image_usage) as f64 / total_memory as f64;

        BackpressureStats {
            active_workers: *self.active_workers.lock().unwrap(),
            worker_limit: *self.current_worker_limit.lock().unwrap(),
            queue_size: self.work_queue.lock().unwrap().len(),
            memory_pressure,
            total_processed: *self.total_processed.lock().unwrap(),
            rejected_due_to_pressure: *self.rejected_due_to_pressure.lock().unwrap(),
        }
    }

    pub fn start_adaptive_controller(&self) {
        let controller = self.clone();

        thread::spawn(move || loop {
            controller.adjust_worker_limits();
            thread::sleep(Duration::from_secs(5));
        });
    }
}

#[derive(Debug, Clone)]
pub struct BackpressureStats {
    pub active_workers: usize,
    pub worker_limit: usize,
    pub queue_size: usize,
    pub memory_pressure: f64,
    pub total_processed: usize,
    pub rejected_due_to_pressure: usize,
}

impl Clone for BackpressureController {
    fn clone(&self) -> Self {
        Self {
            cache: self.cache.clone(),
            prefetcher: self.prefetcher.clone(),
            work_queue: self.work_queue.clone(),
            active_workers: self.active_workers.clone(),
            max_concurrent_workers: self.max_concurrent_workers,
            memory_pressure_threshold: self.memory_pressure_threshold,
            queue_pressure_threshold: self.queue_pressure_threshold,
            current_worker_limit: self.current_worker_limit.clone(),
            last_adjustment: self.last_adjustment.clone(),
            total_processed: self.total_processed.clone(),
            rejected_due_to_pressure: self.rejected_due_to_pressure.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backpressure_rejection() {
        let cache = Arc::new(CacheRegistry::new());
        let prefetcher = Arc::new(IntentAwarePrefetcher::new(cache.clone()));
        let controller = BackpressureController::new(cache, prefetcher);

        let work_item = WorkItem {
            page_id: 1,
            work_type: WorkType::SemanticExtraction,
            priority: 0.8,
            created_at: Instant::now(),
            backpressure_sensitive: true,
        };

        // Should accept work initially
        assert!(controller.submit_work(work_item.clone()).is_ok());

        // Get stats
        let stats = controller.get_backpressure_stats();
        assert_eq!(stats.queue_size, 1);
    }

    #[test]
    fn test_worker_limit_adjustment() {
        let cache = Arc::new(CacheRegistry::new());
        let prefetcher = Arc::new(IntentAwarePrefetcher::new(cache.clone()));
        let controller = BackpressureController::new(cache, prefetcher);

        let initial_limit = *controller.current_worker_limit.lock().unwrap();
        controller.adjust_worker_limits();

        // Should not change significantly under normal conditions
        let new_limit = *controller.current_worker_limit.lock().unwrap();
        assert!((new_limit as i32 - initial_limit as i32).abs() <= 1);
    }
}
