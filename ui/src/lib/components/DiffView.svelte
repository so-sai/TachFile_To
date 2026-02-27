<script lang="ts">
    import { appState } from "../state.svelte";
    import { UI } from "../messages.vi";
    import { CheckCircle, RotateCcw } from "lucide-svelte";

    const kindLabel: Record<string, string> = {
        Added: UI.compare_added,
        Removed: UI.compare_removed,
        Modified: UI.compare_modified,
    };

    const kindColor: Record<string, string> = {
        Added: "text-[var(--color-success)] bg-emerald-50",
        Removed: "text-[var(--color-error)] bg-red-50",
        Modified: "text-[var(--color-warn)] bg-amber-50",
    };

    const report = $derived(appState.diffReport);
</script>

<div class="flex-1 flex flex-col overflow-hidden">
    <!-- Report header -->
    <div
        class="shrink-0 flex items-center justify-between px-6 h-14 border-b border-[var(--color-border)] bg-[var(--color-surface)]"
    >
        <div class="flex items-center gap-3">
            {#if report?.isIdentical}
                <CheckCircle size={16} class="text-[var(--color-success)]" />
                <p class="text-[13px] font-bold text-[var(--color-success)]">
                    {UI.compare_identical}
                </p>
            {:else if report}
                <p class="text-[13px] font-bold text-[var(--color-text)]">
                    <span class="mono font-black text-[var(--color-action)]"
                        >{report.totalDeltas}</span
                    >
                    {" "}{UI.compare_changes}
                </p>
            {/if}
        </div>
        <button onclick={() => appState.reset()} class="btn-ghost">
            <RotateCcw size={13} /> So sánh tệp mới
        </button>
    </div>

    <!-- Delta table -->
    <div class="flex-1 overflow-y-auto scrollable">
        {#if report && !report.isIdentical}
            <table class="w-full text-[12px] border-collapse">
                <thead>
                    <tr
                        class="bg-[var(--color-base)] border-b border-[var(--color-border)] sticky top-0"
                    >
                        <th class="text-left px-4 py-2.5 section-label w-24"
                            >{UI.compare_modified}</th
                        >
                        <th class="text-left px-4 py-2.5 section-label"
                            >{UI.compare_location}</th
                        >
                        <th class="text-left px-4 py-2.5 section-label w-40"
                            >{UI.compare_old}</th
                        >
                        <th class="text-left px-4 py-2.5 section-label w-40"
                            >{UI.compare_new}</th
                        >
                    </tr>
                </thead>
                <tbody>
                    {#each report.deltas as delta, i}
                        <tr
                            class="border-b border-[var(--color-border)] hover:bg-[var(--color-base)] transition-colors"
                        >
                            <td class="px-4 py-2.5">
                                <span
                                    class="badge {kindColor[delta.kind] ?? ''}"
                                >
                                    {kindLabel[delta.kind] ?? delta.kind}
                                </span>
                            </td>
                            <td
                                class="px-4 py-2.5 mono text-[var(--color-text-2)] max-w-[280px] truncate"
                                title={delta.location}>{delta.location}</td
                            >
                            <td
                                class="px-4 py-2.5 mono text-[var(--color-error)]"
                                >{delta.oldValue ?? "—"}</td
                            >
                            <td
                                class="px-4 py-2.5 mono text-[var(--color-success)]"
                                >{delta.newValue ?? "—"}</td
                            >
                        </tr>
                    {/each}
                </tbody>
            </table>
        {:else if report?.isIdentical}
            <div class="flex items-center justify-center h-full opacity-50">
                <CheckCircle size={48} class="text-[var(--color-success)]" />
            </div>
        {/if}
    </div>
</div>
