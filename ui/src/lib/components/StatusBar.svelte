<script lang="ts">
    import { UI } from "../messages.vi";
    import { Wifi, WifiOff } from "lucide-svelte";
</script>

<footer
    class="col-span-2 flex items-center justify-between px-6 border-t border-[var(--color-border)] bg-[var(--color-base)] shrink-0 select-none"
>
    <div class="flex items-center gap-1.5">
        <WifiOff size={11} class="text-[var(--color-text-3)]" />
        <span
            class="text-[10px] font-semibold text-[var(--color-text-3)] uppercase tracking-wider"
        >
            {UI.statusbar_offline}
        </span>
    </div>
    <div class="flex items-center gap-3">
        <span class="text-[10px] font-bold mono text-[var(--color-text-3)]">
            © 2026 SO-SAI
        </span>
        <span class="text-[var(--color-border)]">·</span>
        <span class="text-[10px] mono text-[var(--color-text-3)]"
            >{UI.statusbar_version}</span
        >
    </div>
</footer>
