<script lang="ts">
  import { appState } from '../state.svelte';
  import { UI } from '../messages.vi';
  import { FileText, FileSpreadsheet } from 'lucide-svelte';
</script>

<aside class="flex flex-col border-r border-[var(--color-border)] bg-[var(--color-surface)] overflow-hidden">
  <!-- Sidebar header -->
  <div class="flex items-center justify-between px-4 h-12 border-b border-[var(--color-border)] shrink-0 bg-[var(--color-base)]">
    <span class="section-label">{UI.sidebar_title}</span>
    <span class="mono text-[11px] font-bold text-[var(--color-text-3)]">
      {appState.processedFiles.length}
    </span>
  </div>

  <!-- File list -->
  <div class="flex-1 overflow-y-auto scrollable p-2 space-y-1">
    {#if appState.processedFiles.length === 0}
      <div class="flex flex-col items-center justify-center h-full gap-3 opacity-40 py-12">
        <FileText size={32} color="var(--color-text-3)" />
        <p class="section-label text-center">{UI.sidebar_empty}</p>
      </div>
    {:else}
      {#each appState.processedFiles as file}
        {@const isActive = appState.activeSummaryId === file.id}
        {@const isPdf = file.sourcePath.toLowerCase().endsWith('.pdf')}
        <button
          onclick={() => { appState.activeSummaryId = file.id; }}
          class="w-full text-left px-3 py-2.5 rounded transition-all flex items-start gap-3 cursor-pointer
            {isActive
              ? 'bg-[var(--color-action-50)] border border-blue-200'
              : 'hover:bg-[var(--color-base)] border border-transparent'}"
        >
          <div class="mt-0.5 shrink-0 w-5 h-5 flex items-center justify-center
            {isActive ? 'text-[var(--color-action)]' : 'text-[var(--color-text-3)]'}">
            {#if isPdf}
              <FileText size={14} strokeWidth={2} />
            {:else}
              <FileSpreadsheet size={14} strokeWidth={2} />
            {/if}
          </div>
          <div class="min-w-0 flex-1">
            <p class="text-[12px] font-semibold truncate
              {isActive ? 'text-[var(--color-action)]' : 'text-[var(--color-text)]'}">
              {file.sourcePath.split(/[/\\]/).pop() ?? file.id}
            </p>
            <p class="text-[10px] mono text-[var(--color-text-3)] mt-0.5">
              {file.totalPages} {UI.result_pages}
              {#if file.hasOcr} · OCR{/if}
            </p>
          </div>
        </button>
      {/each}
    {/if}
  </div>
</aside>
