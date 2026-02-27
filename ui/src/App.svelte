<script lang="ts">
  import { appState } from "./lib/state.svelte";
  import Header from "./lib/components/Header.svelte";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import StatusBar from "./lib/components/StatusBar.svelte";
  import DropZone from "./lib/components/DropZone.svelte";
  import ProcessingView from "./lib/components/ProcessingView.svelte";
  import ResultView from "./lib/components/ResultView.svelte";
  import CompareDropZone from "./lib/components/CompareDropZone.svelte";
  import DiffView from "./lib/components/DiffView.svelte";
  import "./app.css";
</script>

<!--
  TachFileTo V1.0 — App Root
  Layout: 2-col grid (sidebar | main) + header + statusbar
  Source of truth: SYSTEM_ARCHITECTURE.md Section 2 (Layer Diagram)
-->
<div class="app-grid">
  <!-- Row 1, col 1–2 -->
  <Header />

  <!-- Row 2, col 1: Sidebar -->
  <Sidebar />

  <!-- Row 2, col 2: Main Canvas — routes by mode + phase -->
  <main class="overflow-hidden flex flex-col bg-[var(--color-base)]">
    {#if appState.mode === "clean"}
      {#if appState.phase === "idle" || appState.phase === "error"}
        <DropZone />
      {:else if appState.phase === "processing"}
        <ProcessingView />
      {:else if appState.phase === "ready"}
        <ResultView />
      {:else if appState.phase === "exporting"}
        <ProcessingView />
      {/if}
    {:else}
      <!-- Compare mode -->
      {#if appState.phase === "idle" || appState.phase === "error"}
        <CompareDropZone />
      {:else if appState.phase === "processing" || appState.phase === "comparing"}
        <ProcessingView />
      {:else if appState.phase === "ready" && appState.diffReport !== null}
        <DiffView />
      {:else}
        <CompareDropZone />
      {/if}
    {/if}
  </main>

  <!-- Row 3, col 1–2 -->
  <StatusBar />
</div>
