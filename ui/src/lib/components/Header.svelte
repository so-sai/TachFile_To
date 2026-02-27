<script lang="ts">
  import { appState } from '../state.svelte';
  import { UI } from '../messages.vi';
  import { Zap, GitCompare } from 'lucide-svelte';

  function setMode(mode: 'clean' | 'compare') {
    appState.setMode(mode);
  }
</script>

<header class="col-span-2 flex items-center justify-between px-6 border-b border-[var(--color-border)] bg-[var(--color-surface)] z-20 select-none shrink-0">
  <!-- Brand -->
  <div class="flex items-center gap-4">
    <div class="flex items-center gap-2.5">
      <div class="w-8 h-8 bg-[var(--color-action)] flex items-center justify-center">
        <Zap size={16} color="white" strokeWidth={2.5} />
      </div>
      <div class="flex flex-col leading-none">
        <span class="text-[14px] font-black tracking-tight text-[var(--color-text)]">
          {UI.app_name}
        </span>
        <span class="text-[9px] font-bold text-[var(--color-text-3)] tracking-[0.15em] uppercase">
          {UI.app_tagline}
        </span>
      </div>
    </div>

    <!-- Mode Switcher -->
    <div class="flex items-center border border-[var(--color-border)] overflow-hidden ml-4">
      <button
        onclick={() => setMode('clean')}
        class="px-4 py-2 text-[11px] font-bold uppercase tracking-wider transition-all cursor-pointer
          {appState.mode === 'clean'
            ? 'bg-[var(--color-action)] text-white'
            : 'bg-[var(--color-surface)] text-[var(--color-text-2)] hover:bg-[var(--color-base)]'}"
      >
        {UI.mode_clean}
      </button>
      <div class="w-px h-5 bg-[var(--color-border)]"></div>
      <button
        onclick={() => setMode('compare')}
        class="px-4 py-2 text-[11px] font-bold uppercase tracking-wider transition-all flex items-center gap-1.5 cursor-pointer
          {appState.mode === 'compare'
            ? 'bg-[var(--color-action)] text-white'
            : 'bg-[var(--color-surface)] text-[var(--color-text-2)] hover:bg-[var(--color-base)]'}"
      >
        <GitCompare size={12} />
        {UI.mode_compare}
      </button>
    </div>
  </div>

  <!-- Status indicator -->
  <div class="flex items-center gap-2">
    {#if appState.phase === 'processing' || appState.phase === 'comparing' || appState.phase === 'exporting'}
      <div class="flex items-center gap-2">
        <div class="w-1.5 h-1.5 rounded-full bg-[var(--color-action)] animate-pulse"></div>
        <span class="text-[11px] font-semibold text-[var(--color-action)]">{UI.status_processing}</span>
      </div>
    {:else if appState.phase === 'ready'}
      <div class="flex items-center gap-2">
        <div class="w-1.5 h-1.5 rounded-full bg-[var(--color-success)]"></div>
        <span class="text-[11px] font-semibold text-[var(--color-success)]">{UI.status_done}</span>
      </div>
    {:else if appState.phase === 'error'}
      <div class="flex items-center gap-2">
        <div class="w-1.5 h-1.5 rounded-full bg-[var(--color-error)]"></div>
        <span class="text-[11px] font-semibold text-[var(--color-error)]">{UI.status_failed}</span>
      </div>
    {:else}
      <div class="flex items-center gap-2">
        <div class="w-1.5 h-1.5 rounded-full bg-[var(--color-success)]"></div>
        <span class="text-[11px] font-semibold text-[var(--color-text-3)]">{UI.status_ready}</span>
      </div>
    {/if}
  </div>
</header>
