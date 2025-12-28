# 🧠 TACHFILETO: ORIENTATION (FOR FUTURE ME & AI AGENTS)

**Version:** 3.0.0 (Iron Core V3.0 - Smart Headers)  
**Last Updated:** 2025-12-26  
**Status:** ✅ SINGLE SOURCE OF TRUTH

---

## 📂 1. Documentation Map (Bản Đồ Chỉ Đường)

Nếu có bất kỳ sự xung đột nào giữa các tài liệu, thứ tự ưu tiên sẽ là:

1. **`GUIDE.md`** (Chính là file này): Định hướng chiến lược và quy tắc AI.
2. **`specs/MASTER_V3.0_DASHBOARD.md`**: Trạng thái thực thi kỹ thuật và Roadmap hiện tại.
3. **`specs/TRUTH_CONTRACT_V1.md`**: Giao ước dữ liệu Rust ↔ React.
4. **`specs/archive/`**: Nghĩa địa của các ý tưởng cũ (Không được sử dụng để code).

---

## 🤖 2. Rules for AI Agents (Lá Chắn Chống Hallucination)

Dành cho các AI Agent tham gia vào dự án này:

### ❌ KHÔNG ĐƯỢC PHÉP

- **Tự ý thay đổi Stack**: Không đề xuất nâng cấp React, Rust, Polars hoặc bất kỳ dependency nào mà không có sự đồng ý của Founder.
- **Giả định Cloud/SaaS**: Không code Login, Cloud Sync, hay bất kỳ tính năng nào trong **Non-Goals (V2.5 Scope Only)**.
- **Khôi sinh Python Worker**: Python đã bị loại bỏ hoàn toàn. Không được đề xuất Python dưới mọi hình thức.
- **Viết code Tiếng Anh cho UI**: Người dùng cuối là QS Việt Nam → 100% tiếng Việt.

### ✅ PHẢI TUÂN THỦ

- **Ngôn ngữ**: 100% tiếng Việt cho UI, thông báo, và error messages.
- **Truth Contract**: Rust PHẢI tính toán logic, React CHỈ hiển thị.
- **Deterministic**: Không AI/ML, chỉ thuật toán rõ ràng.
- **Đọc Spec trước khi code**: Luôn xem `MASTER_V3.0_DASHBOARD.md` trước khi đề xuất thay đổi.

---

## 🏗️ 3. Current Reality (Thực Tại Công Nghệ)

Dự án đã thực hiện các thay đổi "đại phẫu" để đạt được sự tinh khiết:

### Technology Stack

| Layer | Technology | Version | Purpose |
|-------|-----------|---------|---------|
| **Frontend** | React + TypeScript | **19** | Dual-persona UI (Founder + QS) |
| **Desktop Runtime** | Tauri | 2.0 | Native desktop wrapper |
| **Backend** | Rust | Edition 2024 | Iron Core business logic |
| **Data Engine** | Polars | 0.52 | DataFrame processing (1M+ rows) |
| **Excel Parser** | Calamine | 0.32 | Universal .xls/.xlsx support |
| **Smart Headers** | Iron Core V3.0 | - | Fuzzy detection + merged cells |
| **IPC** | Tauri Commands | - | Direct Rust ↔ React |

### Architectural Purge (Làm sạch Lịch sử)

**ĐÃ TIÊU HỦY**:
- ❌ `backend/` directory (Python Worker)
- ❌ `crates/` directory (Old Rust architecture)
- ❌ Stdio JSON IPC (thay bằng Tauri Commands)
- ❌ SQLite Cache (load toàn bộ vào RAM)
- ❌ Legacy font converters (TCVN3/VNI) - tạm hoãn đến V2.6

**LÝ DO**: Đơn giản hóa để tập trung vào **Dashboard + Virtual Ledger** trước.

---

## 🎯 4. What We're Building (V2.5 Scope)

### Persona 1: Founder (Dashboard View)

**Câu hỏi**: "Có nguy hiểm không? Lỗ bao nhiêu?"

**Giao diện**:
- 🚦 Status Light: XANH/VÀNG/ĐỎ (deterministic rules)
- 💰 Financial Overview: Contract value, paid, projected profit
- ⚠️ Top Risks: Max 5 items sorted by cost impact
- 📋 Pending Actions: Prioritized by urgency

**Design**: Brutalist (hard edges, bold colors, zero ambiguity)

### Persona 2: QS/PM (Data View)

**Câu hỏi**: "Dòng nào sai? Sai vì sao?"

**Giao diện**:
- 📊 Virtual Ledger: Infinite scroll (1M+ rows)
- 🔍 Column Normalization: Auto-standardized Vietnamese terms
- 📏 Tabular Numbers: Aligned for easy scanning

**Design**: Excel-like (native scrollbar, 32px rows, enterprise density)

---

## 🚫 5. Explicit Non-Goals (V2.5 Scope Only)

The following features are intentionally excluded from V2.5,  
even though they exist in the long-term roadmap:

- ❌ **Multi-project aggregation** (planned V2.9+)
- ❌ **Historical trend analysis** (planned V2.8+)
- ❌ **Cloud sync or login system** (post V3.0)
- ❌ **PDF table extraction** (V2.6 - Docling integration)
- ❌ **Visual evidence viewer** (V2.7 - Evidence panel)
- ❌ **Mobile companion app** (V2.9+)

**Reason**:  
V2.5 focuses exclusively on **single-project, deterministic validation**  
to establish founder trust in the core decision engine.

---

## 🗂️ 6. Project Structure (Iron Core Era)

```
TachFile_To/
├── ui/
│   ├── src/                        # React 19 Frontend
│   │   ├── App.tsx                 # Tab Navigation (Dashboard | Data View)
│   │   ├── components/
│   │   │   ├── DashboardMockup.tsx # Founder Dashboard (Brutalist UI)
│   │   │   └── VirtualLedger/      # QS Data View (TanStack Virtual)
│   │   └── styles/                 # Enterprise Eye-Safe Design
│   └── src-tauri/                  # Rust Backend (SINGLE SOURCE)
│       ├── Cargo.toml              # Polars 0.52 + calamine 0.32
│       ├── src/
│       │   ├── main.rs             # Tauri entry
│       │   ├── lib.rs              # Command registry
│       │   ├── excel_engine.rs     # Excel reading + Normalization
│       │   ├── normalizer.rs       # Vietnamese term standardization
│       │   └── dashboard.rs        # Deterministic business rules
│       └── target/
└── docs/
    ├── GUIDE.md                    # THIS FILE
    ├── CHANGELOG.md                # Version history
    ├── LESSONS_LEARNED.md          # Founder notes
    └── specs/
        ├── MASTER_V3.0_DASHBOARD.md   # Technical spec (V3.0)
        ├── TRUTH_CONTRACT_V1.md       # Iron Core ↔ UI Schema
        ├── IPC_PROTOCOL.md            # (Legacy - for reference)
        └── archive/                   # Legacy specs (v1.x, v2.1-2.5)
```

---

## 🚀 7. Quick Start (Development)

### Running the Application

```bash
cd ui
npm run tauri dev

# App will:
# 1. Start Vite dev server (port 1420)
# 2. Compile Rust backend
# 3. Launch Tauri window with Dashboard tab active
```

### Testing with Real Data

1. Click "Dashboard" tab (default)
2. Drag & drop Excel file (.xlsx)
3. Iron Core processes and returns `ProjectTruth`
4. Dashboard renders status, risks, actions
5. Switch to "Data View" tab for drill-down

### Building for Production

```bash
cd ui
npm run tauri build

# Output: ui/src-tauri/target/release/tachfileto-core.exe
```

### Running Tests

```bash
cd ui/src-tauri
cargo test --lib

# Expected: 33/33 tests PASSING
```

---

## 🧪 8. Testing Status

**Rust Unit Tests**: ✅ 100% PASSING (33/33 tests)

```bash
$ cargo test --lib
running 33 tests
test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured
```

**Test Categories**:
1. Dashboard Business Logic (17 tests)
2. Column Header Normalizer (6 tests)
3. Terminology Normalizer (10 tests)

---

## 📊 9. Performance Targets

| Metric | Target | Actual (V2.5) |
|--------|--------|---------------|
| Excel load (100k rows) | <2s | 1.2s |
| Dashboard calculation | <500ms | 156ms |
| UI render (initial) | <100ms | 68ms |
| Memory usage (1M rows) | <500MB | 380MB |
| Binary size | <15MB | 12.4MB |

---

## 🛡️ 10. Tại sao tài liệu này lại quan trọng?

### Chống tự bắn vào chân
Sau 3 tháng không sờ vào code, bạn sẽ quên tại sao mình chọn ngưỡng rủi ro 15% thay vì 20%. `GUIDE.md` sẽ nhắc bạn lý do.

### Quản lý Agent
Khi bạn reset context hoặc dùng một Agent mới, chỉ cần yêu cầu nó:  
*"Đọc GUIDE.md và MASTER_V2.5 để nắm bắt thực tại"*.  
Nó sẽ không dám "tư vấn láo" về Python hay Cloud nữa.

### Kỷ luật Solo-Dev
Bạn đang làm việc mà 70% startup chỉ làm khi đã trả giá quá đắt.  
Việc hệ thống hóa ngay từ đầu giúp dự án của bạn có tầm vóc của một Enterprise ngay cả khi chỉ có một người làm.

---

## 🔗 11. External References

- [React 19 Documentation](https://react.dev/)
- [Tauri 2.0 Framework](https://tauri.app/)
- [Polars Documentation](https://pola-rs.github.io/polars/)
- [Calamine (Excel Parser)](https://docs.rs/calamine/)

---

## 📝 12. Version History

| Version | Date | Changes |
|---------|------|---------|
| 3.0.0 | 2025-12-26 | Iron Core V3.0: Smart Header Detection, Merged Cell Propagation, Fuzzy Matching, UI Purification |
| 2.5.0 | 2025-12-26 | Iron Core orientation, React 19, Non-Goals V2.5 (Archived) |
| 2.4.0 | 2025-12-25 | Polars 0.52 upgrade, Calamine 0.32 |
| 2.3.0 | 2025-12-24 | Pure Rust stack, removed Python |
| 1.1.0 | 2025-12-25 | ARCHIVED - Python Worker era |

---

**For detailed technical specifications, navigate to:**
- [MASTER_V3.0_DASHBOARD.md](file:///e:/DEV/TachFile_To/docs/specs/MASTER_V3.0_DASHBOARD.md) - Technical spec
- [TRUTH_CONTRACT_V1.md](file:///e:/DEV/TachFile_To/docs/specs/TRUTH_CONTRACT_V1.md) - Data contract
