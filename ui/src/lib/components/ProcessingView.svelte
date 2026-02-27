<script lang="ts">
    import { appState } from "../state.svelte";
    import { UI } from "../messages.vi";
    import { Loader2 } from "lucide-svelte";
</script>

<div
    class="flex-1 flex flex-col items-center justify-center gap-8 p-12 bg-[var(--color-base)]"
>
    <!-- Spinner -->
    <div
        class="w-12 h-12 flex items-center justify-center text-[var(--color-action)] animate-spin"
    >
        <Loader2 size={36} strokeWidth={1.5} />
    </div>

    <!-- Phase label -->
    <div class="text-center space-y-2">
        <p class="text-[15px] font-semibold text-[var(--color-text)]">
            {appState.progress.label || UI.status_processing}
        </p>
        {#if appState.progress.total > 0}
            <p class="text-[12px] mono text-[var(--color-text-3)]">
                {UI.progress_page}
                {appState.progress.current}
                {UI.progress_of}
                {appState.progress.total}
            </p>
        {/if}
    </div>

    <!-- Progress bar -->
    <div class="progress-track w-80">
        <div
            class="progress-fill"
            style="width: {appState.progress.total > 0
                ? Math.round(
                      (appState.progress.current / appState.progress.total) *
                          100,
                  )
                : 30}%"
        ></div>
    </div>

    <!-- Cancel button — always accessible (elite-ui-rust: State Lock Prevention) -->
    <button
        onclick={() => appState.reset()}
        class="btn-ghost text-[var(--color-text-3)] hover:text-[var(--color-error)]"
    >
        {UI.action_cancel}
    </button>
</div>
