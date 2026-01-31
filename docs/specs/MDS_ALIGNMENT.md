# TachFileTo – MDS Alignment Document (Updated V1.2)

**MDS Reference:** MDS-ELITE9-2026-v1.2
**Status:** SPECIALIZED PRECISION INGESTION ORGAN (OPTIONAL)
**Governance Tier:** Tier 2 (Sidecar / Plugin)
**Law Compliance:** Strict LAW-04 Adherence

---

## I. ECOSYSTEM ROLE (RE-ALIGNED)

> **MDS Definition:** TachFileTo là **Specialized Organ**. Nó độc lập về thực thi nhưng không nằm trong Critical Path của hệ sinh thái.

* **TachFileTo tồn tại để:** Giải quyết Chaos (PDF nặng/scan/rác) mà AutoQSVN không xử lý được.
* **Hệ sinh thái tồn tại để:** Vận hành bình thường ngay cả khi TachFileTo bị gỡ bỏ.

---

## II. THE SEPARATION OF POWERS (QUYỀN LỰC VÀ THỰC THI)

Để tránh vi phạm **LAW-01 (SSOT)** và **LAW-02 (No Reverse Dependency)**, ranh giới được phân định rõ:

| Thực thể | Vai trò (Role) | Thẩm quyền (Authority) |
| --- | --- | --- |
| **TachFileTo** | **Truth Shaper** | Trích xuất, Chuẩn hóa, Khám phá (Detect & Declare). |
| **iron_core** | **Truth Holder** | Phê duyệt, Lưu trữ, Thực thi (Approve & Enforce). |

---

## III. UPDATED LAW COMPLIANCE (STRICT)

* **LAW-01 (SSOT):** TachFileTo không giữ sự thật. Nó chỉ tạo ra **Pre-Truth Artefacts** (Vật phẩm tiền-sự thật) dưới dạng JSON sạch để gửi cho Judge (iron_core).
* **LAW-04 (Optionality):** Tuyệt đối không đưa thẩm mỹ (1.5px, Neon-Yellow) vào Luật. Các yếu tố này được phân loại là **UI Preference**, không phải kiến trúc.
* **Storage Rule:** Cấm **Persistent Storage**. Chỉ cho phép **Ephemeral Artefacts** (Vật phẩm tạm thời) trong quá trình xử lý (Double-pass/Audit). Janitor phải dọn sạch sau khi Task hoàn thành.

---

## IV. CAPABILITY VS. CONSTITUTION

Chúng ta tách biệt rõ ràng giữa **Năng lực** và **Hiến pháp** để tránh "Monolith trá hình":

1. **Capability (Implementation):**
* Sử dụng Python 3.14t No-GIL cho hiệu năng cao.
* Sử dụng Rust để đảm bảo Determinism.
* *Lưu ý: Nếu thay đổi công nghệ, vai trò Ingestion vẫn không đổi.*

2. **Constitution (Governance):**
* **Detect & Declare:** Chỉ báo cáo vi phạm (như Veto-rate > 20%).
* **NO Enforcement:** Không có quyền tự chặn (Veto) dữ liệu nạp vào hệ sinh thái nếu Judge chưa ký duyệt.

---

### 🎯 KẾT LUẬN

TachFileTo là một công cụ **chuyên dụng, độc lập và vô hại** đối với sự tồn vong của hệ thống lớn.

> **"TachFileTo biết rất nhiều, làm rất khỏe, nhưng không có quyền quyết định số phận của bất kỳ dữ liệu nào."**

---

## VII. REFERENCES

- [GOVERNANCE/MDS.md](file:///e:/DEV/elite10-ecosystem/GOVERNANCE/MDS.md) – Master Design Spec
- [GOVERNANCE/ARCH_LAWS.md](file:///e:/DEV/elite10-ecosystem/GOVERNANCE/ARCH_LAWS.md) – Architecture Laws
- [docs/BOUNDARY_MANIFEST.md](file:///e:/DEV/elite_9_VN-ecosystem/app-tool-TachFileTo/docs/BOUNDARY_MANIFEST.md) – Local boundaries

---

**Document Authority:** Elite 10 Ecosystem Architecture Board

