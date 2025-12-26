# TRUTH CONTRACT V1.0 - IRON CORE ↔ DASHBOARD

**Version:** 1.0.0  
**Last Updated:** 2025-12-26  
**Status:** SPECIFICATION (Pre-Implementation)

---

## 🎯 Purpose

This document defines the **immutable contract** between:
- **Iron Core (Rust)**: Data processing engine
- **Dashboard UI (React)**: Founder decision interface

> **Critical Rule**: UI has NO logic. UI only renders what Iron Core declares as truth.

---

## 📊 JSON Schema: `ProjectTruth`

### Root Object

```typescript
interface ProjectTruth {
  // === META ===
  project_name: string;              // "CHUNG CƯ TACHFILETO - ĐỢT 4"
  last_updated: string;              // ISO 8601: "2025-12-26T12:00:00+07:00"
  data_source: string;               // "file.xlsx" or "database"
  
  // === OVERALL STATUS ===
  project_status: "SAFE" | "WARNING" | "CRITICAL";
  status_reason: string;             // Human-readable explanation
  
  // === FINANCIAL OVERVIEW ===
  financials: {
    contract_value: number;          // VND (raw number, UI formats)
    paid_to_date: number;            // VND
    payment_percent: number;         // 0-100
    projected_profit: number;        // VND (can be negative)
    profit_margin_percent: number;   // -100 to 100
    unapproved_amount: number;       // VND (pending approval)
  };
  
  // === DEVIATION ANALYSIS ===
  deviation: {
    total_percent: number;           // Overall deviation (-100 to 100)
    high_risk_count: number;         // Items with >10% deviation
    critical_count: number;          // Items with >20% deviation
    total_items: number;             // Total line items analyzed
  };
  
  // === TOP RISKS (Max 5) ===
  top_risks: Array<{
    id: number;
    item_code: string;               // "AF.109978"
    item_name: string;               // "Thép D12"
    deviation_value: number;         // Absolute difference
    deviation_percent: number;       // Percentage
    deviation_unit: string;          // "t", "m3", "VND"
    reason: string;                  // "Vượt thiết kế", "Sai đơn giá"
    severity: "HIGH" | "MEDIUM" | "LOW";
    estimated_cost_impact: number;   // VND
  }>;
  
  // === ACTIONABLE ITEMS (Max 5) ===
  pending_actions: Array<{
    id: number;
    action: string;                  // "Ký phụ lục thép D12"
    priority: "URGENT" | "HIGH" | "NORMAL";
    responsible: string;             // "QS", "PM", "Founder"
    deadline: string | null;         // ISO 8601 or null
  }>;
  
  // === METRICS ===
  metrics: {
    total_rows_processed: number;
    normalized_columns: number;
    data_quality_score: number;      // 0-100
    last_calculation_time_ms: number;
  };
}
```

---

## 🚦 Status Determination Rules (Iron Core Logic)

### SAFE (Green)
```rust
if deviation.total_percent < 5.0 
   && deviation.critical_count == 0 
   && financials.profit_margin_percent > 10.0
{
    status = "SAFE"
}
```

### WARNING (Yellow)
```rust
if (deviation.total_percent >= 5.0 && deviation.total_percent < 15.0)
   || (deviation.critical_count > 0 && deviation.critical_count < 5)
   || (financials.profit_margin_percent > 0.0 && financials.profit_margin_percent <= 10.0)
{
    status = "WARNING"
}
```

### CRITICAL (Red)
```rust
if deviation.total_percent >= 15.0
   || deviation.critical_count >= 5
   || financials.profit_margin_percent <= 0.0
{
    status = "CRITICAL"
}
```

---

## 📝 Example JSON Response

```json
{
  "project_name": "CHUNG CƯ TACHFILETO - ĐỢT 4",
  "last_updated": "2025-12-26T12:00:00+07:00",
  "data_source": "du_toan_dot4.xlsx",
  
  "project_status": "CRITICAL",
  "status_reason": "Lệch vượt 15% và lợi nhuận âm",
  
  "financials": {
    "contract_value": 19000000000,
    "paid_to_date": 12400000000,
    "payment_percent": 65.3,
    "projected_profit": -240000000,
    "profit_margin_percent": -1.26,
    "unapproved_amount": 450000000
  },
  
  "deviation": {
    "total_percent": 12.4,
    "high_risk_count": 8,
    "critical_count": 3,
    "total_items": 247
  },
  
  "top_risks": [
    {
      "id": 1,
      "item_code": "AF.109978",
      "item_name": "Thép D12",
      "deviation_value": 5.2,
      "deviation_percent": 18.3,
      "deviation_unit": "t",
      "reason": "Vượt thiết kế do thay đổi kết cấu",
      "severity": "HIGH",
      "estimated_cost_impact": 78000000
    },
    {
      "id": 2,
      "item_code": "AF.109985",
      "item_name": "Cát vàng",
      "deviation_value": 60000,
      "deviation_percent": 8.5,
      "deviation_unit": "VND/m3",
      "reason": "Sai đơn giá so với hợp đồng",
      "severity": "MEDIUM",
      "estimated_cost_impact": 45000000
    }
  ],
  
  "pending_actions": [
    {
      "id": 1,
      "action": "Ký phụ lục thép D12 (QS đang chờ)",
      "priority": "URGENT",
      "responsible": "Founder",
      "deadline": "2025-12-27T17:00:00+07:00"
    },
    {
      "id": 2,
      "action": "Gửi biên bản nghiệm thu móng A1-A5",
      "priority": "HIGH",
      "responsible": "QS",
      "deadline": null
    }
  ],
  
  "metrics": {
    "total_rows_processed": 247,
    "normalized_columns": 8,
    "data_quality_score": 92.5,
    "last_calculation_time_ms": 156
  }
}
```

---

## 🔒 Contract Guarantees

### Iron Core MUST:
1. ✅ Return valid JSON matching this schema
2. ✅ Calculate `project_status` using deterministic rules
3. ✅ Provide `status_reason` explaining the decision
4. ✅ Sort `top_risks` by `estimated_cost_impact` (descending)
5. ✅ Sort `pending_actions` by `priority` then `deadline`
6. ✅ Return data within 500ms for files <100k rows

### Dashboard UI MUST NOT:
1. ❌ Calculate percentages
2. ❌ Determine status colors
3. ❌ Filter or sort risks
4. ❌ Infer business logic
5. ❌ Cache stale data

---

## 🚀 Implementation Phases

### Phase 1 (Current - V2.5)
- ✅ Dashboard renders MOCK_DATA
- ✅ UI layout finalized
- ⏳ Iron Core implements schema

### Phase 2 (V2.6)
- ⏳ Rust command: `get_project_truth() -> ProjectTruth`
- ⏳ Dashboard calls Tauri invoke
- ⏳ Remove MOCK_DATA

### Phase 3 (V2.7+)
- ⏳ Real-time updates on data change
- ⏳ Export to Word/PDF
- ⏳ Drill-down from Dashboard → Data View

---

## 📚 Related Documents

- [ARCHITECTURE_V2.5.md](./ARCHITECTURE_V2.5.md)
- [Dashboard Mockup](../../ui/src/components/DashboardMockup.tsx)
- [Iron Core Implementation](../../ui/src-tauri/src/dashboard.rs)

---

**This contract is IMMUTABLE once approved. Changes require version bump (V2.0).**
