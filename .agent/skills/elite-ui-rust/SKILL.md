---
name: elite-ui-rust
description: Elite Standards for Rust/React/Tailwind v4 UI Development (AEC Edition)
version: 1.1.0
---

# 🛡️ Elite UI/UX Rust Protocol (The "Clean Hands" Doctrine)

This skill encapsulates the battle-hardened standards for building industrial-grade user interfaces using **Tauri (Rust)**, **React 19**, and **Tailwind CSS v4**. It is forged from the "so-sai" ecosystem experience.

## I. The "Clean Hands" Doctrine
**"If it is unused, it must die."**

1.  **Dead Code Zero Tolerance**:
    -   NEVER keep CSS classes "just in case" (e.g., `btn-primary`, `rigid-border` if not used).
    -   If a linter or build tool complains about syntax, first check if the code is even *needed*. deleting > fixing.
    -   **Audit Rule**: Before every release build, grep for custom classes in `src/`. If 0 hits, remove from `index.css`.

2.  **No "Ghost" Windows**:
    -   Production builds MUST NOT show a command prompt / terminal window.
    -   **Fix**: Add `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` to `src-tauri/src/main.rs`.

3.  **Import Sanitation**:
    -   Duplicate or unused imports in `.tsx` or `.rs` files block the production pipeline (`tsc` or `cargo`).
    -   **Action**: Use `cargo fix --workspace --allow-dirty` for Rust and always verify `npx tsc --noEmit` for React before building.

## II. Tailwind CSS v4 Engine Mode
Recovering from v3 habits is critical.

1.  **Initialization**:
    -   **DO NOT USE**: `@tailwind base;` / `@tailwind components;`
    -   **MUST USE**: `@import "tailwindcss";`

2.  **No Invalid Utilities**:
    -   **Avoid**: `@utility btn:hover`. Colons are illegal in utility names.
    -   **Use Nesting**:
        ```css
        @utility btn-primary {
          /* styles */
          &:hover { /* styles */ }
        }
        ```

3.  **Scoped Isolation**:
    -   Do not pollute global scope with scrollbar hacks.
    -   **YES**: `.custom-class { &::-webkit-scrollbar { ... } }`
    -   **NO**: `::-webkit-scrollbar { ... }` (Global pollution)

## III. AEC Design Constitution (Architectural Standard)
Software for Engineers must feel like a precision instrument.

1.  **The Rigid Frame**:
    -   Layouts MUST NOT shift pixel-by-pixel.
    -   Use `scrollbar-gutter: stable` or `overflow: hidden` on body.
    -   **Dimensions**:
        -   Sidebar: `280px` (Fixed)
        -   Header: `56px` to `64px`
        -   Status Bar (Refined Action Bar): `40px`

2.  **Typography of Truth**:
    -   **Inter**: For User Interface, Labels, Instructions. (Human readable)
    -   **JetBrains Mono**: For **DATA**, **NUMBERS**, **HASHES**. (Machine readable, monospaced alignment).
    -   *Never mix them up.*

3.  **Color Semantics**:
    -   **Action Blue**: Primary operations (`#2563EB`).
    -   **Alert Orange**: Non-fatal warnings (`#F59E0B`).
    -   **Forensic Gray**: Backgrounds and borders (`#E5E7EB`, `#F9FAFB`). Avoid pure black `#000`.

## IV. Production Hygiene (The Build Loop)
1.  **Capability Schema Validation**:
    -   Tauri v2 uses strict permission identifiers (e.g., `fs:default`, `fs:read-all`).
    -   **Warning**: Avoid using non-standard identifiers like `fs:allow-read-recursive` which trigger `exit code: 1` during build.

2.  **Clean Build Cycle**:
    -   Run `cargo clean` if linking errors occur.
    -   **Command**: `npm run tauri build`.

3.  **Installer Verification**:
    -   Check `target/release/bundle/msi` or `nsis`.
    -   Verify file size > 5MB.

## V. The Scrub Protocol (V.1.1.0 Evolution)
**"A clean engine leaves no digital fingerprints."**

1.  **Runtime Hygiene**:
    -   Implement **AEC-CRASH-DUMP**: Error boundaries must offer a one-click state dump (JSON) for forensic debugging without developer console access.
2.  **Post-Build Sanitation**:
    -   Automatically clean up stale `target/` artifacts if they exceed the project threshold (recommend 5GB).
3.  **Privacy Purge**:
    -   Ensure all `blob:` URLs and temp buffers from PDF/Excel parsing are revoked/dropped immediately after render to prevent persistent memory footprints.
