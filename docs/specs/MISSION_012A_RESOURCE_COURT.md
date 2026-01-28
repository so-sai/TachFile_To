# MISSION 012A - ResourceCourt (Phán Quyết Tài Nguyên)

**Version:** 1.0.0-SKELETON  
**Date:** 2026-01-28  
**Status:** ✅ Phase 1 Complete - Policy Engine Ready  
**Next Phase:** 012B - Executioner (Thực Thi An Toàn)

---

## 🎯 Executive Summary

Mission 012A triển khai **Tam Quyền Phân Lập (Separation of Powers)** để quản lý tài nguyên cache của TachFileTo:

| Thành Phần | Trách Nhiệm | Quyền Hạn |
|-----------|-----------|----------|
| **CacheRegistry** | Thống kê dữ liệu | Quan sát, không quyết định |
| **ResourceCourt** | Phán xét chính sách | Tính điểm, ra phán quyết |
| **Executioner** | Thực hiện lệnh | Xóa, ghi log (Phase 2) |

---

## 📊 Kiến Trúc

```
┌─────────────────────────────────────────┐
│  CacheRegistry (TAM THỊ)                │
│  • Theo dõi file size, age, access      │
│  • Không có logic xóa                   │
│  • Facts only - Stateless               │
└────────────┬────────────────────────────┘
             │
             ↓ (entries + metrics)
┌─────────────────────────────────────────┐
│  ResourceCourt (TOÀ ÁN)                 │
│  • Tính Eviction Score (4 biến)         │
│  • Render Verdict (RETAIN/MONITOR/...)  │
│  • Audit Log                            │
└────────────┬────────────────────────────┘
             │
             ↓ (verdicts)
┌─────────────────────────────────────────┐
│  Executioner (HÀN LẦM) - Phase 2        │
│  • Soft-delete (quiesce)                │
│  • Hard-delete (irreversible)           │
│  • Recovery log                         │
└─────────────────────────────────────────┘
```

---

## 🧮 Eviction Score Formula

Phương trình toán học:

$$EvictionScore = w_1 \cdot Size + w_2 \cdot Age + w_3 \cdot Viewport + w_4 \cdot Entropy$$

Trong đó:

- **$w_1 = 0.25$**: Size Component - tỉ lệ dung lượng so với limit
- **$w_2 = 0.25$**: Age Component - tuổi file (normalized by 30 days)
- **$w_3 = 0.30$**: Viewport Component - khoảng cách từ viewport
- **$w_4 = 0.20$**: Entropy Component - mật độ file (nguy hiểm cho filesystem)

**Severity Mapping:**

```rust
if score >= 0.8 => CRITICAL  (Hard delete nếu cache quá)
if score >= 0.6 => HIGH      (Soft delete - có thể khôi phục)
if score >= 0.4 => MEDIUM    (Monitor)
else            => LOW       (Retain)
```

---

## 🔐 Policy Configuration (Hiến Pháp)

Default policy được thiết kế cho **Desktop (8-16GB RAM)**:

```rust
EvictionPolicy {
    max_cache_size_bytes: 500 * 1024 * 1024,  // 500 MB
    min_age_seconds: 86400,                    // 1 day
    size_weight: 0.25,
    age_weight: 0.25,
    viewport_weight: 0.30,
    entropy_weight: 0.20,
    entropy_high_file_count: 10000,            // 10k files threshold
    entropy_warning_threshold: 0.6,
    eviction_threshold_critical: 0.8,
    eviction_threshold_high: 0.6,
    eviction_threshold_medium: 0.4,
    max_files_per_directory: 50000,
    purge_all_enabled: false,                  // Tuyệt đối không auto
}
```

**Tại sao**:
- **500MB limit**: Hợp lý cho cache trên Desktop SSD
- **1 day min age**: Tránh xóa file vừa download
- **High entropy weight**: Filesystem bị bào mòn bởi 50k+ PNG files nhỏ
- **purge_all_enabled=false**: Bảo vệ dữ liệu người dùng

---

## 📋 Domain Models

### CacheEntry
```rust
pub struct CacheEntry {
    pub file_id: String,
    pub file_path: String,
    pub file_size_bytes: u64,
    pub file_count: usize,           // Number of files in entry
    pub created_at: u64,             // UNIX timestamp
    pub last_accessed_at: u64,
    pub access_count: u64,
    pub user_pinned: bool,           // User protection
    pub viewport_distance: f64,      // 0.0 = in view, 1.0 = far
}
```

### EvictionScore
```rust
pub struct EvictionScore {
    pub file_id: String,
    pub size_component: f64,         // w1 * size_ratio
    pub age_component: f64,          // w2 * age_ratio
    pub viewport_component: f64,     // w3 * viewport_distance
    pub entropy_component: f64,      // w4 * entropy_factor
    pub total_score: f64,            // Weighted sum (0.0 to 1.0)
    pub severity_level: EvictionSeverity,
}
```

### EvictionVerdict
```rust
pub struct EvictionVerdict {
    pub file_id: String,
    pub action: EvictionAction,      // RETAIN | MONITOR | SOFT_DELETE | HARD_DELETE
    pub reason: String,
    pub score: f64,
    pub timestamp: u64,
    pub is_reversible: bool,
}
```

---

## ✅ Test Coverage

Skeleton đã bao gồm **6 core test cases**:

1. ✅ `test_registry_basic_operations` - Registry có thể track entries
2. ✅ `test_court_eviction_score_calculation` - Score được tính chính xác
3. ✅ `test_court_judgment_with_pinned_entry` - User protection hoạt động
4. ✅ `test_entropy_calculation` - Entropy factor đúng
5. ✅ `test_multiple_entries_judgment` - Court xử lý batch entries
6. (Placeholder cho Phase 2) Executioner tests

**Run tests**:
```bash
cd e:\DEV\elite_9_VN-ecosystem\app-tool-TachFileTo
cargo test --lib --package tachfileto-bin resource_court -- --nocapture
```

---

## 🛡️ Key Design Principles

### 1. Separation of Powers
- ❌ CacheRegistry **KHÔNG** có quyền xóa file
- ❌ ResourceCourt **KHÔNG** trực tiếp tương tác filesystem
- ✅ Executioner chỉ xóa những gì có Warrant từ Court

### 2. Determinism
- Score luôn được tính theo công thức xác định
- Không có random, không có heuristic "thần kỳ"
- Audit log ghi lại mọi quyết định

### 3. User Protection
- `user_pinned` là tuyệt đối → RETAIN không có điều kiện
- Items trong viewport (distance < 0.1) + accessed > 5 times → RETAIN
- Hard delete chỉ khi cache quá limit

### 4. No Auto-Purge-All
- `purge_all_enabled = false` theo mặc định
- Manual purge yêu cầu explicit action + confirmation
- Irreversible operations phải "in your face" không lặng lẽ

---

## 🗓️ Phase Roadmap

### Phase 1: ✅ COMPLETE
- [x] Define domain models
- [x] Implement CacheRegistry (observation)
- [x] Implement ResourceCourt (judgment)
- [x] EvictionScore formula
- [x] Unit tests (6 tests passing)

### Phase 2: 012B - Executioner (TBD)
- [ ] Quiesce protocol (stop all readers)
- [ ] Soft-delete implementation
- [ ] Hard-delete with recovery log
- [ ] Transaction semantics (Two-Phase Commit for PurgeAll)
- [ ] Integrateion tests with real filesystem

### Phase 3: 012C - Idle-Aware Maintenance (TBD)
- [ ] CPU/Disk/Engine idle signal detection
- [ ] Incremental SQLite vacuum
- [ ] Background sanitation without user latency
- [ ] Zero user-visible pause

---

## 🚀 Integration Points (Chuẩn Bị)

Khi ready, ResourceCourt sẽ integrate vào:

```rust
// In excel_engine.rs
pub struct CacheManager {
    registry: CacheRegistry,
    court: ResourceCourt,
    // executioner: Executioner,  // Phase 2
}

impl CacheManager {
    pub fn make_room(&mut self, needed_bytes: u64) {
        let current_size = self.registry.total_size_bytes();
        if current_size + needed_bytes > LIMIT {
            // 1. Court judges all entries
            let verdicts = self.court.judge_entries(&self.registry, current_size);
            
            // 2. Sort by severity + score
            // 3. Execute verdicts (Phase 2)
        }
    }
}
```

---

## ⚠️ Critical Gaps (To Be Addressed)

### For Mission 012B (Executioner):
1. **Soft vs Hard Delete Logic** - What's recoverable? When?
2. **Quiesce Protocol** - How to safely delete while readers active?
3. **Transaction Semantics** - Atomic purge_all with rollback?

### For Mission 012C (Idle Maintenance):
1. **Idle Signal** - CPU < 15%? Disk I/O quiet? Engine suspended?
2. **Vacuum Strategy** - Incremental vs full?
3. **Latency Contract** - Max pause time?

---

## 📞 Contact & Questions

**Implemented by:** GitHub Copilot  
**For:** TachFileTo Project (so-sai)  
**Date:** 2026-01-28

Nếu có thắc mắc:
- Xem code comments trong `resource_court.rs`
- Run tests để hiểu behavior
- Tính toán examples bằng tay với công thức

**"Hệ thống này không chỉ đúng khi chạy, mà còn đúng khi bị bỏ quên."** 🛡️🚀🦀
