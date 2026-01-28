use crate::cache_registry::{CacheRegistry, ImageBlock, SemanticBlock};
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct UserIntent {
    pub current_page: u32,
    pub scroll_velocity: f64,       // pixels per second
    pub viewport_range: (u32, u32), // (start_page, end_page)
    pub last_updated: u64,
}

#[derive(Debug)]
pub struct PrefetchRequest {
    pub page_id: u32,
    pub priority: f64,
    pub request_type: PrefetchType,
}

#[derive(Debug, Clone)]
pub enum PrefetchType {
    Semantic,
    Image,
    Both,
}

#[derive(Debug)]
pub struct IntentAwarePrefetcher {
    cache: Arc<CacheRegistry>,
    user_intent: Arc<Mutex<UserIntent>>,
    prefetch_queue: Arc<Mutex<VecDeque<PrefetchRequest>>>,
    worker_cv: Arc<Condvar>,

    // Prefetch control parameters
    max_queue_size: usize,
    prefetch_batch_size: usize,
    prefetch_interval: Duration,

    // Intent decay factors
    velocity_weight: f64,
    proximity_weight: f64,
    recency_weight: f64,
}

impl IntentAwarePrefetcher {
    pub fn new(cache: Arc<CacheRegistry>) -> Self {
        Self {
            cache: cache.clone(),
            user_intent: Arc::new(Mutex::new(UserIntent {
                current_page: 0,
                scroll_velocity: 0.0,
                viewport_range: (0, 0),
                last_updated: current_timestamp(),
            })),
            prefetch_queue: Arc::new(Mutex::new(VecDeque::new())),
            worker_cv: Arc::new(Condvar::new()),
            max_queue_size: 50,
            prefetch_batch_size: 5,
            prefetch_interval: Duration::from_millis(200),
            velocity_weight: 0.4,
            proximity_weight: 0.3,
            recency_weight: 0.3,
        }
    }

    pub fn update_user_intent(
        &self,
        current_page: u32,
        scroll_velocity: f64,
        viewport_range: (u32, u32),
    ) {
        let mut intent = self.user_intent.lock().unwrap();
        intent.current_page = current_page;
        intent.scroll_velocity = scroll_velocity;
        intent.viewport_range = viewport_range;
        intent.last_updated = current_timestamp();

        // Trigger prefetch calculation
        self.calculate_priorities();
    }

    fn calculate_priorities(&self) {
        let intent = self.user_intent.lock().unwrap();
        let mut queue = self.prefetch_queue.lock().unwrap();

        // Clear old priorities
        queue.clear();

        // Calculate priority for pages around current position
        let current = intent.current_page;
        let (viewport_start, viewport_end) = intent.viewport_range;

        // Prefetch window: 10 pages behind, 20 pages ahead (adjustable)
        let prefetch_start = if current > 10 { current - 10 } else { 0 };
        let prefetch_end = current + 20;

        for page_id in prefetch_start..=prefetch_end {
            if page_id == current {
                continue;
            } // Skip current page

            // Skip if already cached
            if self.cache.get_semantic(page_id).is_some() && self.cache.get_image(page_id).is_some()
            {
                continue;
            }

            let priority = self.calculate_page_priority(page_id, &intent);
            if priority > 0.1 {
                // Minimum threshold
                let request_type = self.determine_request_type(page_id);

                queue.push_back(PrefetchRequest {
                    page_id,
                    priority,
                    request_type,
                });
            }
        }

        // Sort by priority (highest first)
        queue
            .make_contiguous()
            .sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());

        // Limit queue size
        while queue.len() > self.max_queue_size {
            queue.pop_front();
        }
    }

    fn calculate_page_priority(&self, page_id: u32, intent: &UserIntent) -> f64 {
        let now = current_timestamp();
        let time_factor = 1.0 - ((now - intent.last_updated) as f64 / 3600.0).min(1.0); // Decay over 1 hour

        // 1. Proximity to current page
        let distance = (page_id as i32 - intent.current_page as i32).abs() as f64;
        let proximity_score = if distance == 0.0 {
            1.0
        } else {
            1.0 / (1.0 + distance * 0.1)
        };

        // 2. Scroll velocity prediction
        let velocity_score = if intent.scroll_velocity > 0.0 {
            // Predict future position based on velocity
            let predicted_offset = intent.scroll_velocity * 2.0; // 2 seconds prediction
            let predicted_page = (intent.current_page as f64 + predicted_offset).max(0.0);
            let predicted_distance = (page_id as f64 - predicted_page).abs();
            if predicted_distance == 0.0 {
                1.0
            } else {
                1.0 / (1.0 + predicted_distance * 0.05)
            }
        } else {
            // Static reading: prioritize nearby pages
            proximity_score
        };

        // 3. Viewport alignment bonus
        let viewport_score =
            if page_id >= intent.viewport_range.0 && page_id <= intent.viewport_range.1 {
                1.0
            } else {
                let viewport_distance = if page_id < intent.viewport_range.0 {
                    (intent.viewport_range.0 - page_id) as f64
                } else {
                    (page_id - intent.viewport_range.1) as f64
                };
                if viewport_distance == 0.0 {
                    1.0
                } else {
                    1.0 / (1.0 + viewport_distance * 0.2)
                }
            };

        // Weighted combination
        let priority = (self.velocity_weight * velocity_score
            + self.proximity_weight * proximity_score
            + self.recency_weight * viewport_score)
            * time_factor;

        priority.max(0.0).min(1.0)
    }

    fn determine_request_type(&self, page_id: u32) -> PrefetchType {
        let has_semantic = self.cache.get_semantic(page_id).is_some();
        let has_image = self.cache.get_image(page_id).is_some();

        match (has_semantic, has_image) {
            (false, false) => PrefetchType::Both,
            (false, true) => PrefetchType::Semantic,
            (true, false) => PrefetchType::Image,
            (true, true) => PrefetchType::Both, // Shouldn't reach here
        }
    }

    pub fn start_prefetch_worker<F>(&self, fetch_fn: F)
    where
        F: Fn(u32, PrefetchType) -> Result<(), String> + Send + Sync + 'static,
    {
        let cache = self.cache.clone();
        let queue = self.prefetch_queue.clone();
        let cv = self.worker_cv.clone();
        let batch_size = self.prefetch_batch_size;
        let interval = self.prefetch_interval;

        thread::spawn(move || {
            loop {
                let requests = {
                    let mut q = queue.lock().unwrap();
                    let mut batch = Vec::new();

                    // Take up to batch_size highest priority requests
                    for _ in 0..batch_size {
                        if let Some(req) = q.pop_front() {
                            batch.push(req);
                        } else {
                            break;
                        }
                    }

                    batch
                };

                if requests.is_empty() {
                    // Wait for new requests
                    let _guard = cv.wait(queue.lock().unwrap()).unwrap();
                    continue;
                }

                // Process batch
                for request in requests {
                    // Check backpressure before fetching
                    match request.request_type {
                        PrefetchType::Semantic | PrefetchType::Both => {
                            if !cache.can_accept_semantic_work() {
                                continue;
                            }
                        }
                        PrefetchType::Image | PrefetchType::Both => {
                            if !cache.can_accept_image_work() {
                                continue;
                            }
                        }
                    }

                    // Execute fetch
                    if let Err(e) = fetch_fn(request.page_id, request.request_type) {
                        eprintln!("Prefetch error for page {}: {}", request.page_id, e);
                    }
                }

                // Brief pause between batches
                thread::sleep(interval);
            }
        });
    }

    pub fn get_prefetch_stats(&self) -> PrefetchStats {
        let queue = self.prefetch_queue.lock().unwrap();
        let intent = self.user_intent.lock().unwrap();
        let (semantic_usage, image_usage) = self.cache.get_memory_stats();

        PrefetchStats {
            queue_size: queue.len(),
            current_page: intent.current_page,
            scroll_velocity: intent.scroll_velocity,
            semantic_memory_usage: semantic_usage,
            image_memory_usage: image_usage,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PrefetchStats {
    pub queue_size: usize,
    pub current_page: u32,
    pub scroll_velocity: f64,
    pub semantic_memory_usage: usize,
    pub image_memory_usage: usize,
}

impl Drop for IntentAwarePrefetcher {
    fn drop(&mut self) {
        // Notify worker to wake up and exit
        self.worker_cv.notify_all();
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_calculation() {
        let cache = Arc::new(CacheRegistry::new());
        let prefetcher = IntentAwarePrefetcher::new(cache);

        let intent = UserIntent {
            current_page: 10,
            scroll_velocity: 0.0,
            viewport_range: (8, 12),
            last_updated: current_timestamp(),
        };

        // Test proximity - current page should have highest priority
        let priority_current = prefetcher.calculate_page_priority(10, &intent);
        let priority_near = prefetcher.calculate_page_priority(11, &intent);
        let priority_far = prefetcher.calculate_page_priority(20, &intent);

        assert!(priority_current > priority_near);
        assert!(priority_near > priority_far);
    }

    #[test]
    fn test_velocity_prediction() {
        let cache = Arc::new(CacheRegistry::new());
        let prefetcher = IntentAwarePrefetcher::new(cache);

        let intent_fast_scroll = UserIntent {
            current_page: 10,
            scroll_velocity: 5.0, // Fast scrolling
            viewport_range: (8, 12),
            last_updated: current_timestamp(),
        };

        let intent_static = UserIntent {
            current_page: 10,
            scroll_velocity: 0.0, // Static reading
            viewport_range: (8, 12),
            last_updated: current_timestamp(),
        };

        // Fast scrolling should give higher priority to pages ahead
        let priority_fast_15 = prefetcher.calculate_page_priority(15, &intent_fast_scroll);
        let priority_static_15 = prefetcher.calculate_page_priority(15, &intent_static);

        assert!(priority_fast_15 > priority_static_15);
    }
}
