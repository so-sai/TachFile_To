<script lang="ts">
    import { appState } from "../state.svelte";
    import { UI } from "../messages.vi";
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import { UploadCloud, GitCompare } from "lucide-svelte";
    import type { DocumentSummary, ProcessError } from "../types";

    async function pickFile(slot: "a" | "b") {
        const selected = await open({
            multiple: false,
            filters: [{ name: "Tài liệu", extensions: ["pdf", "docx"] }],
        });
        if (typeof selected !== "string") return;

        try {
            const summary: DocumentSummary = await invoke("process_document", {
                path: selected,
            });
            appState.setCompareSlot(slot, summary);
        } catch (err) {
            const code =
                (err as { code?: ProcessError })?.code ?? "EnginePanic";
            appState.setError(code as ProcessError);
        }
    }

    async function runCompare() {
        if (!appState.compareFileA || !appState.compareFileB) return;
        appState.startComparing();
        try {
            const report = await invoke("compare_documents", {
                idA: appState.compareFileA.id,
                idB: appState.compareFileB.id,
            });
            appState.finishComparing(report as any);
        } catch (err) {
            const code =
                (err as { code?: ProcessError })?.code ?? "EnginePanic";
            appState.setError(code as ProcessError);
        }
    }

    function slotLabel(slot: "a" | "b") {
        return slot === "a" ? UI.compare_file_a : UI.compare_file_b;
    }

    function slotSummary(slot: "a" | "b") {
        return slot === "a" ? appState.compareFileA : appState.compareFileB;
    }
</script>

<div
    class="flex-1 flex flex-col gap-6 p-8 bg-[var(--color-base)] overflow-y-auto scrollable"
>
    <div class="grid grid-cols-2 gap-4">
        {#each ["a", "b"] as const as slot}
            {@const summary = slotSummary(slot)}
            <div
                class="border border-dashed border-[var(--color-border-2)] p-6 flex flex-col items-center gap-4 bg-[var(--color-surface)]"
            >
                <p class="section-label">{slotLabel(slot)}</p>

                {#if summary}
                    <div class="text-center">
                        <p
                            class="text-[13px] font-semibold text-[var(--color-text)] truncate max-w-[180px]"
                        >
                            {summary.sourcePath.split(/[/\\]/).pop()}
                        </p>
                        <p
                            class="text-[11px] mono text-[var(--color-text-3)] mt-1"
                        >
                            {summary.totalPages}
                            {UI.result_pages}
                        </p>
                    </div>
                    <div
                        class="w-3 h-3 rounded-full bg-[var(--color-success)]"
                    ></div>
                {:else}
                    <div class="text-[var(--color-text-3)]">
                        <UploadCloud size={28} strokeWidth={1.5} />
                    </div>
                    <button
                        onclick={() => pickFile(slot)}
                        class="btn-primary text-[11px] px-3 py-2"
                    >
                        {UI.action_choose_file}
                    </button>
                {/if}
            </div>
        {/each}
    </div>

    <div class="flex justify-center">
        <button
            onclick={runCompare}
            disabled={!appState.compareFileA ||
                !appState.compareFileB ||
                appState.phase === "comparing"}
            class="btn-primary gap-2 px-6 py-3"
        >
            <GitCompare size={15} />
            {UI.action_compare}
        </button>
    </div>
</div>
