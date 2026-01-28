// =========================================================================
// MISSION 011: PARALLEL DISPATCHER - SYSTEM LAYER OPTIMIZATION
// =========================================================================
// Rayon-based parallel processing for multi-page documents
// Tuân thủ SYSTEM_CONTRACT_V1.2.1: Tách nhỏ PDF thành các mảnh

use crate::engine_context::{EngineContext, OperatingMode};
use crate::mupdf_text::{EliteTextPage, FzStextPage};
use crate::semantic::engine::SemanticEngine;
use rayon::prelude::*;
use std::sync::Arc;

/// Dispatch Engine xử lý đa luồng theo Operating Mode
pub struct ParallelDispatcher {
    engine_context: Arc<EngineContext>,
    /// Chế độ vận hành quyết định chiến lược parallel
    mode: OperatingMode,
}

impl ParallelDispatcher {
    pub fn new(engine_context: Arc<EngineContext>) -> Self {
        let mode = {
            let ctx_ref = engine_context.as_ref();
            match &ctx_ref.mode {
                OperatingMode::Normal { .. } => OperatingMode::Normal {
                    ram_gb: ctx_ref.mode.clone().unwrap().ram_gb,
                    logical_cores: ctx_ref.mode.clone().unwrap().logical_cores,
                    simd_enabled: ctx_ref.mode.clone().unwrap().simd_enabled,
                },
                OperatingMode::Limp { .. } => OperatingMode::Limp {
                    reason: "System degraded".to_string(),
                    remaining_features: vec!["Single-threaded".to_string()],
                },
            }
        };

        Self {
            engine_context,
            mode,
        }
    }

    /// Xử lý toàn bộ document với chiến lược phù hợp
    pub fn process_document(
        &self,
        doc: &crate::EliteDocument,
    ) -> Result<Vec<String>, crate::Error> {
        let page_count = doc.count_pages()?;
        println!(
            "🚀 TachFileTo: Processing {} pages in {:?} mode",
            page_count, self.mode
        );

        match &self.mode {
            OperatingMode::Normal {
                logical_cores,
                simd_enabled,
                ..
            } => self.process_normal_mode(doc, page_count, *logical_cores, *simd_enabled),
            OperatingMode::Limp { .. } => self.process_limp_mode(doc, page_count),
        }
    }

    /// NORMAL MODE: Xử lý đa luồng tối ưu
    fn process_normal_mode(
        &self,
        doc: &crate::EliteDocument,
        page_count: i32,
        logical_cores: u32,
        simd_enabled: bool,
    ) -> Result<Vec<String>, crate::Error> {
        // Quyết định số worker threads (không vượt quá số trang)
        let num_workers = std::cmp::min(logical_cores, page_count as u32);
        println!(
            "   🛡️ Using {} worker threads for parallel processing",
            num_workers
        );

        // Tạo vector indices để phân chia công việc
        let page_indices: Vec<i32> = (0..page_count).collect();

        // Rayon parallel processing
        let results: Result<Vec<_>, _> = page_indices
            .par_iter() // Chuyển sang parallel iterator
            .with_min_len(1) // Đảm bảo mỗi worker có ít nhất 1 trang
            .map(|page_index| {
                // Mỗi worker có EngineContext riêng thông qua Arc
                let local_engine = SemanticEngine::new(&self.engine_context);

                // Xử lý trang đơn lẻ
                match local_engine.process_page_with_context(page_index, &self.engine_context) {
                    Ok(markdown) => {
                        println!(
                            "   ✅ Page {} processed: {} chars",
                            page_index + 1,
                            markdown.len()
                        );
                        Ok(markdown)
                    }
                    Err(e) => {
                        eprintln!("   ❌ Page {} failed: {:?}", page_index + 1, e);
                        Err(e)
                    }
                }
            })
            .collect();

        match results {
            Ok(markdowns) => {
                println!(
                    "   🎉 Parallel processing complete: {} pages",
                    markdowns.len()
                );
                Ok(markdowns)
            }
            Err(e) => {
                // Nếu một page thất bại, trả về error
                Err(crate::Error::ParallelProcessingFailed(format!(
                    "Parallel processing failed: {:?}",
                    e
                )))
            }
        }
    }

    /// LIMP MODE: Xử lý tuần tự để tiết kiệm tài nguyên
    fn process_limp_mode(
        &self,
        doc: &crate::EliteDocument,
        page_count: i32,
    ) -> Result<Vec<String>, crate::Error> {
        println!("   ⚠️  LIMP MODE: Processing pages sequentially...");

        let mut markdowns = Vec::with_capacity(page_count as usize);

        for page_index in 0..page_count {
            let local_engine = SemanticEngine::new(&self.engine_context);

            match local_engine.process_page_with_context(page_index, &self.engine_context) {
                Ok(markdown) => {
                    println!("   ✅ Page {} processed (LIMP)", page_index + 1);
                    markdowns.push(markdown);
                }
                Err(e) => {
                    eprintln!("   ❌ Page {} failed (LIMP): {:?}", page_index + 1, e);
                    return Err(crate::Error::LimpModeProcessingFailed(format!(
                        "Page {} failed in limp mode: {:?}",
                        page_index + 1,
                        e
                    )));
                }
            }
        }

        println!(
            "   🐢 Sequential processing complete: {} pages",
            markdowns.len()
        );
        Ok(markdowns)
    }

    /// Thống kê hiệu năng
    pub fn get_performance_stats(
        &self,
        total_pages: i32,
        processing_time_ms: u64,
    ) -> PerformanceStats {
        let pages_per_second = (total_pages as f64) / (processing_time_ms as f64 / 1000.0);

        let (strategy, cores_used) = match &self.mode {
            OperatingMode::Normal { logical_cores, .. } => ("Parallel".to_string(), *logical_cores),
            OperatingMode::Limp { .. } => ("Sequential".to_string(), 1),
        };

        PerformanceStats {
            strategy,
            cores_used,
            pages_per_second,
            total_pages,
            processing_time_ms,
        }
    }
}

/// Thống kê hiệu năng để báo cáo
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub strategy: String,
    pub cores_used: u32,
    pub pages_per_second: f64,
    pub total_pages: i32,
    pub processing_time_ms: u64,
}
