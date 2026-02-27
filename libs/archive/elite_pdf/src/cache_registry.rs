use dashmap::DashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct SemanticBlock {
    pub page_id: u32,
    pub content: String,
    pub bbox_metadata: Vec<(f32, f32, f32, f32)>, // x0, y0, x1, y1
    pub last_accessed: u64,
    pub is_verified: bool,
}

#[derive(Debug, Clone)]
pub struct ImageBlock {
    pub page_id: u32,
    pub png_path: String,
    pub render_dpi: u16,
    pub file_size: usize,
    pub last_accessed: u64,
}

#[derive(Debug, Clone)]
pub enum CacheEntry {
    Semantic(SemanticBlock),
    Image(ImageBlock),
}

#[derive(Debug)]
pub struct CacheRegistry {
    // L1 Semantic Cache - DashMap for Lock-free sharding
    semantic_cache: Arc<DashMap<u32, SemanticBlock>>,
    semantic_memory_usage: Arc<Mutex<usize>>, // in bytes

    // L2 Image Cache - DashMap for LRU & Memory-aware
    image_cache: Arc<DashMap<u32, ImageBlock>>,
    image_memory_usage: Arc<Mutex<usize>>, // in bytes

    // Configuration
    max_semantic_memory: usize, // 100MB default
    max_image_memory: usize,    // 500MB default
}

impl CacheRegistry {
    pub fn new() -> Self {
        Self {
            semantic_cache: Arc::new(DashMap::new()),
            semantic_memory_usage: Arc::new(Mutex::new(0)),
            image_cache: Arc::new(DashMap::new()),
            image_memory_usage: Arc::new(Mutex::new(0)),
            max_semantic_memory: 100 * 1024 * 1024, // 100MB
            max_image_memory: 500 * 1024 * 1024,    // 500MB
        }
    }

    pub fn get_semantic(&self, page_id: u32) -> Option<SemanticBlock> {
        if let Some(mut block) = self.semantic_cache.get_mut(&page_id) {
            block.last_accessed = current_timestamp();
            Some(block.clone())
        } else {
            None
        }
    }

    pub fn put_semantic(&self, block: SemanticBlock) -> Result<(), String> {
        let page_id = block.page_id;

        // Calculate memory impact
        let content_size = block.content.len();
        let bbox_size = block.bbox_metadata.len() * 16;
        let total_size = content_size + bbox_size + 64;

        // Check if we need to evict
        {
            let usage = *self.semantic_memory_usage.lock().unwrap();
            if usage + total_size > self.max_semantic_memory {
                self.evict_semantic_lru(total_size)?;
            }
        }

        // Insert new block
        {
            let mut usage = self.semantic_memory_usage.lock().unwrap();
            if let Some(old_block) = self.semantic_cache.get(&page_id) {
                let old_size = old_block.content.len() + old_block.bbox_metadata.len() * 16 + 64;
                *usage = usage.saturating_sub(old_size);
            }
            self.semantic_cache.insert(page_id, block);
            *usage += total_size;
        }

        Ok(())
    }

    pub fn get_image(&self, page_id: u32) -> Option<ImageBlock> {
        if let Some(mut block) = self.image_cache.get_mut(&page_id) {
            block.last_accessed = current_timestamp();
            Some(block.clone())
        } else {
            None
        }
    }

    pub fn put_image(&self, block: ImageBlock) -> Result<(), String> {
        let page_id = block.page_id;
        let size = block.file_size;

        // Check if we need to evict
        {
            let usage = *self.image_memory_usage.lock().unwrap();
            if usage + size > self.max_image_memory {
                self.evict_image_lru(size)?;
            }
        }

        // Insert new block
        {
            let mut usage = self.image_memory_usage.lock().unwrap();
            if let Some(old_block) = self.image_cache.get(&page_id) {
                *usage = usage.saturating_sub(old_block.file_size);
            }
            self.image_cache.insert(page_id, block);
            *usage += size;
        }

        Ok(())
    }

    fn evict_semantic_lru(&self, needed_space: usize) -> Result<(), String> {
        let mut freed_space = 0;
        let mut entries: Vec<_> = self.semantic_cache.iter().map(|r| (*r.key(), r.value().clone())).collect();
        entries.sort_by_key(|(_, block)| block.last_accessed);

        for (page_id, block) in entries {
            if freed_space >= needed_space { break; }
            let now = current_timestamp();
            if block.is_verified && (now - block.last_accessed) > 300 {
                if self.semantic_cache.remove(&page_id).is_some() {
                    let block_size = block.content.len() + block.bbox_metadata.len() * 16 + 64;
                    let mut usage = self.semantic_memory_usage.lock().unwrap();
                    *usage = usage.saturating_sub(block_size);
                    freed_space += block_size;
                }
            }
        }

        if freed_space < needed_space {
            return Err("Cannot free enough semantic memory".to_string());
        }
        Ok(())
    }

    fn evict_image_lru(&self, needed_space: usize) -> Result<(), String> {
        let mut freed_space = 0;
        let mut entries: Vec<_> = self.image_cache.iter().map(|r| (*r.key(), r.clone())).collect();
        entries.sort_by_key(|(_, block)| block.last_accessed);

        for (page_id, block) in entries {
            if freed_space >= needed_space { break; }
            if self.image_cache.remove(&page_id).is_some() {
                let mut usage = self.image_memory_usage.lock().unwrap();
                *usage = usage.saturating_sub(block.file_size);
                freed_space += block.file_size;
            }
        }

        if freed_space < needed_space {
            return Err("Cannot free enough image memory".to_string());
        }
        Ok(())
    }

    // Backpressure hooks for Mission 011
    pub fn can_accept_semantic_work(&self) -> bool {
        let usage = *self.semantic_memory_usage.lock().unwrap();
        usage < (self.max_semantic_memory as f64 * 0.8) as usize // 80% threshold
    }

    pub fn can_accept_image_work(&self) -> bool {
        let usage = *self.image_memory_usage.lock().unwrap();
        usage < (self.max_image_memory as f64 * 0.8) as usize // 80% threshold
    }

    pub fn get_memory_stats(&self) -> (usize, usize) {
        let semantic_usage = *self.semantic_memory_usage.lock().unwrap();
        let image_usage = *self.image_memory_usage.lock().unwrap();
        (semantic_usage, image_usage)
    }

    pub fn clear(&self) {
        self.semantic_cache.clear();
        self.image_cache.clear();
        *self.semantic_memory_usage.lock().unwrap() = 0;
        *self.image_memory_usage.lock().unwrap() = 0;
    }
}

impl Default for CacheRegistry {
    fn default() -> Self {
        Self::new()
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
    fn test_semantic_cache_operations() {
        let registry = CacheRegistry::new();

        let block = SemanticBlock {
            page_id: 1,
            content: "Test content".to_string(),
            bbox_metadata: vec![(0.0, 0.0, 100.0, 100.0)],
            last_accessed: current_timestamp(),
            is_verified: true,
        };

        // Test put and get
        assert!(registry.put_semantic(block.clone()).is_ok());
        let retrieved = registry.get_semantic(1);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, "Test content");
    }

    #[test]
    fn test_image_cache_lru() {
        let registry = CacheRegistry::new();

        let block1 = ImageBlock {
            page_id: 1,
            png_path: "/tmp/page1.png".to_string(),
            render_dpi: 144,
            file_size: 1024,
            last_accessed: current_timestamp(),
        };

        let block2 = ImageBlock {
            page_id: 2,
            png_path: "/tmp/page2.png".to_string(),
            render_dpi: 144,
            file_size: 1024,
            last_accessed: current_timestamp(),
        };

        // Put both blocks
        assert!(registry.put_image(block1).is_ok());
        assert!(registry.put_image(block2).is_ok());

        // Access block1 to make it most recently used
        registry.get_image(1);

        // Check backpressure
        assert!(registry.can_accept_image_work());
    }
}
