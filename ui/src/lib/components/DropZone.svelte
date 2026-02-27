<script lang="ts">
    import { appState } from "../state.svelte";
    import { UI, ERROR_MESSAGES } from "../messages.vi";
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import { UploadCloud, FolderOpen, AlertCircle } from "lucide-svelte";
    import type { DocumentSummary, ProcessError } from "../types";

    let dragOver = $state(false);

    async function handleFile(filePath: string) {
        appState.startProcessing(UI.status_processing);
        try {
            const summary: DocumentSummary = await invoke("process_document", {
                path: filePath,
            });
            const md: string = await invoke("export_markdown", {
                id: summary.id,
            });
            appState.finishProcessing(summary, md);
        } catch (err) {
            const code =
                (err as { code?: ProcessError })?.code ?? "EnginePanic";
            appState.setError(code as ProcessError);
        }
    }

    async function openFilePicker() {
        const selected = await open({
            multiple: false,
            filters: [{ name: "Tài liệu", extensions: ["pdf", "docx"] }],
        });
        if (typeof selected === "string") {
            await handleFile(selected);
        }
    }

    function onDragEnter(e: DragEvent) {
        e.preventDefault();
        dragOver = true;
    }

    function onDragLeave() {
        dragOver = false;
    }

    async function onDrop(e: DragEvent) {
        e.preventDefault();
        dragOver = false;
        const file = e.dataTransfer?.files?.[0];
        if (!file) return;
        // In Tauri, dropped files provide the path via the file object
        const path = (file as File & { path?: string }).path;
        if (path) await handleFile(path);
    }
</script>

<!-- Drop Zone — always accessible, not hidden behind loading (elite-ui-rust: Always-Open Doors) -->
<div
    class="flex-1 flex flex-col items-center justify-center gap-6 p-8
    transition-colors duration-200
    {dragOver ? 'bg-[var(--color-action-50)]' : 'bg-[var(--color-base)]'}"
    ondragover={onDragEnter}
    ondragleave={onDragLeave}
    ondrop={onDrop}
    role="region"
    aria-label="Vùng thả tệp"
>
    <!-- Drop target visual -->
    <div
        class="flex flex-col items-center gap-5 p-12 border-2 border-dashed transition-colors
    {dragOver
            ? 'border-[var(--color-action)]'
            : 'border-[var(--color-border-2)]'}"
    >
        <div
            class="w-14 h-14 flex items-center justify-center bg-[var(--color-action-50)] text-[var(--color-action)]"
        >
            <UploadCloud size={28} strokeWidth={1.5} />
        </div>

        <div class="text-center space-y-1">
            <p class="text-[18px] font-bold text-[var(--color-text)]">
                {UI.dropzone_title}
            </p>
            <p class="text-[13px] text-[var(--color-text-3)]">
                {UI.dropzone_subtitle}
            </p>
        </div>

        <div class="flex items-center gap-3">
            <div class="h-px w-16 bg-[var(--color-border)]"></div>
            <span
                class="text-[11px] text-[var(--color-text-3)] uppercase tracking-wider"
                >{UI.dropzone_or}</span
            >
            <div class="h-px w-16 bg-[var(--color-border)]"></div>
        </div>

        <button onclick={openFilePicker} class="btn-primary gap-2">
            <FolderOpen size={14} />
            {UI.action_choose_file}
        </button>
    </div>

    <!-- Error state message -->
    {#if appState.phase === "error" && appState.error}
        <div
            class="flex items-start gap-3 p-4 border border-[var(--color-error)] bg-red-50 max-w-md"
        >
            <AlertCircle
                size={16}
                class="text-[var(--color-error)] shrink-0 mt-0.5"
            />
            <div>
                <p class="text-[13px] font-semibold text-[var(--color-error)]">
                    {UI.status_failed}
                </p>
                <p class="text-[12px] text-[var(--color-text-2)] mt-0.5">
                    {ERROR_MESSAGES[appState.error]}
                </p>
            </div>
        </div>
    {/if}
</div>
