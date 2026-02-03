# TACHFILETO: RELEASE PROTOCOL RC-1 (STABLE)

> **Protocol ID:** RC-1-PROTOCOL-2026
> **Status:** EXECUTIVE ORDER
> **Target Version:** 1.0.1.Stable

---

## 1. PRE-FLIGHT VERIFICATION
Before any production build is finalized, the following architectural checkpoints MUST be verified:

### 1.1 UI Constitution Compliance
- [ ] No `border-radius` found in compiled CSS.
- [ ] Tabular numbers enabled for all data grids.
- [ ] Brutalist scrollbars (14px) active.
- [ ] Panel 3 successfully toggles between Evidence/History modes.

### 1.2 Forensic Integrity
- [ ] SHA-256 Hash Chaining verified for Ledger of Corrections.
- [ ] `actor` field correctly captures system `USERNAME`.
- [ ] `seal_table_truth` successfully produces a signed snapshot.

---

## 2. THE SEALING CEREMONY
The build process must follow these deterministic steps:

1.  **Code Freeze**: All PRs merged into `main`.
2.  **State Reset**: Clear all local `ForensicState` caches.
3.  **Production Compile**: `npm run tauri build`.
4.  **Hash Verification**: Generate a global SHA-256 hash of the `.msi` / `.exe` installer.

---

## 3. DISTRIBUTION PROTOCOL
- **Installer Naming**: `TachFileTo_v1.0.1_STABLE_RC1.msi`
- **Release Channel**: Forensic-Only (Restricted).
- **Audit Requirement**: Every installation must generate an `INSTALL_ID` for traceability.

---

## 4. EXECUTIVE SIGN-OFF
The build is considered **LOCKED** once the following hash is recorded:
- **Installer SHA-256**: `[PENDING_BUILD_COMPLETION]`

---
**END OF PROTOCOL**
🛡️⚖️🏁
