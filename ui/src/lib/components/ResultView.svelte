<script lang="ts">
    import { appState } from "../state.svelte";
    import { UI } from "../messages.vi";
    import { invoke } from "@tauri-apps/api/core";
    import { writeTextFile, BaseDirectory } from "@tauri-apps/plugin-fs";
    import { save } from "@tauri-apps/plugin-dialog";
    import {
        FileDown,
        Copy,
        CheckCheck,
        RotateCcw,
        FileText,
    } from "lucide-svelte";

    let copied = $state(false);

    async function copyForAI() {
        if (!appState.markdown) return;
        await navigator.clipboard.writeText(appState.markdown);
        copied = true;
        setTimeout(() => {
            copied = false;
        }, 2000);
    }

    async function exportMarkdown() {
        if (!appState.activeSummaryId) return;
        appState.startExporting();
        try {
            const md: string = await invoke("export_markdown", {
                id: appState.activeSummaryId,
            });
            const path = await save({
                defaultPath: "export.md",
                filters: [{ name: "Markdown", extensions: ["md"] }],
            });
            if (path) {
                await writeTextFile(path, md);
            }
        } finally {
            appState.finishExporting();
        }
    }

    function startNew() {
        appState.reset();
    }

    const summary = $derived(appState.activeSummary);
</script>

<div class="flex-1 flex flex-col overflow-hidden">
    <!-- Result toolbar -->
    <div
        class="shrink-0 flex items-center justify-between px-6 h-14 border-b border-[var(--color-border)] bg-[var(--color-surface)]"
    >
        <div class="flex items-center gap-3">
            <FileText size={16} class="text-[var(--color-action)]" />
            <div>
                <p class="text-[13px] font-bold text-[var(--color-text)]">
                    {UI.result_title_clean}
                </p>
                {#if summary}
                    <p class="text-[11px] text-[var(--color-text-3)] mono">
                        {summary.totalPages}
                        {UI.result_pages} ·
                        {summary.hasOcr
                            ? UI.result_ocr_applied
                            : UI.result_no_ocr}
                    </p>
                {/if}
            </div>
        </div>
        <div class="flex items-center gap-2">
            <button onclick={copyForAI} class="btn-ghost">
                {#if copied}
                    <CheckCheck size={13} /> {UI.toast_copied}
                {:else}
                    <Copy size={13} /> {UI.action_copy_ai}
                {/if}
            </button>
            <button
                onclick={exportMarkdown}
                class="btn-ghost"
                disabled={appState.phase === "exporting"}
            >
                <FileDown size={13} />
                {UI.action_export_md}
            </button>
            <button onclick={startNew} class="btn-ghost" title="Xử lý tệp mới">
                <RotateCcw size={13} />
            </button>
        </div>
    </div>

    <!-- Markdown preview -->
    <div class="flex-1 overflow-y-auto scrollable p-6 bg-[var(--color-base)]">
        {#if appState.markdown}
            <pre
                class="mono text-[12px] leading-relaxed text-[var(--color-text)] whitespace-pre-wrap break-words
        max-w-prose mx-auto bg-[var(--color-surface)] p-6 border border-[var(--color-border)]">{appState.markdown}</pre>
        {:else}
            <div class="flex items-center justify-center h-full opacity-30">
                <p class="section-label">{UI.result_markdown}</p>
            </div>
        {/if}
    </div>
</div>
