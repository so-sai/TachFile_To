# Global Rules: Google Antigravity Intelligence Protocol v1.1

These rules apply globally across all workspaces and projects for this user.

## 🔎 Proactive Context Discovery

1. **Mandatory Workspace Scan**: Before executing any technical or document-related task, the agent MUST scan the current directory and its parent hierarchy for a `.agent` folder.
2. **Local Rule Inheritance**: If a `.agent/skills` or `.agent/workflows` directory exists, the agent MUST list their contents and integrate their logic into the current task's execution plan.
3. **Cross-Project Awareness**: Recognize organizational markers (e.g., `DT_BUILDING`) and automatically apply related global identities (e.g., `dt-legal-doc`).

## 🛡️ Identity & Mindset Enforcement

1. **Identity Hierarchy**:
   - Personal User Rules override Global Skills.
   - Project-Specific (Local) Rules override Global Skills.
   - Global "Elite" Skills provide the technical baseline.
2. **Explicit Routing**: Always output a `## Routing Decision` section (as defined in `elite-router`) to demonstrate transparency in skill selection.

## 📋 Document Integrity

- All documents created for this user MUST adhere to the **doc-legal-artifact**, **doc-operational-artifact**, or **doc-marketing-artifact** standards based on their purpose.

## 🧼 Resource Hygiene (CLEAN_OR_DIE)

> **"CẤM tuyệt đối việc tạo file .txt hay .log ở bất kỳ thư mục nào ngoài `/logs` hoặc `/temp`. Mọi file debug phát sinh phải bị xóa ngay sau khi mission kết thúc."**

## 🏁 Data Purity Protocol (The Janitor's Decree)

1. **Stateless & Pure**: Logic layers like `Janitor` must be pure transformers. No state, no semantic inference.
2. **Encoding First**: Every I/O boundary MUST enforce UTF-8 integrity. Mojibake is a critical failure.
3. **Audit Trail**: Transformations must be logged (`JanitorReport`), but reports are non-authoritative for business logic.
