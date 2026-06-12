<script lang="ts">
  import { RefreshCw } from "@lucide/svelte";
  import { kiosk } from "$lib/kiosk/store.svelte";
  import { isTauri } from "$lib/kiosk/bridge";

  const state = $derived.by(() => {
    if (!isTauri()) return { dot: "bg-muted-foreground", label: "Browser-Vorschau" };
    if (kiosk.status?.mock) return { dot: "bg-amber-500", label: "Demo-Modus" };
    if (kiosk.status?.connected)
      return { dot: "bg-success", label: `Verbunden · ${kiosk.status.clientId}` };
    return { dot: "bg-destructive", label: "Nicht verbunden" };
  });

  const lastSyncLabel = $derived(
    kiosk.lastSync
      ? kiosk.lastSync.toLocaleTimeString("de-DE", { hour: "2-digit", minute: "2-digit" })
      : null,
  );

  const busy = $derived(kiosk.syncing || kiosk.processing);
</script>

<footer
  class="h-8 shrink-0 flex items-center gap-4 px-4 bg-surface-chrome backdrop-blur-xl border-t border-hairline text-[11px] text-muted-foreground select-none"
>
  <span class="inline-flex items-center gap-2 min-w-0">
    <span class={`size-1.5 rounded-full shrink-0 ${state.dot}`}></span>
    <span class="truncate">{state.label}</span>
  </span>

  {#if kiosk.status?.connected && kiosk.status.depotId}
    <span class="hidden sm:inline truncate">Depot: {kiosk.status.depotId}</span>
  {/if}
  {#if !kiosk.loading}
    <span class="hidden md:inline tabular-nums">{kiosk.products.length} Apps</span>
  {/if}

  <span class="flex-1"></span>

  {#if lastSyncLabel}
    <span class="hidden sm:inline">Zuletzt synchronisiert: {lastSyncLabel} Uhr</span>
  {/if}

  <button
    type="button"
    class="inline-flex items-center gap-1.5 h-6 px-2 rounded-md font-medium text-foreground hover:bg-accent/60 transition-colors disabled:opacity-50 cursor-pointer"
    disabled={busy || kiosk.loading || !isTauri()}
    onclick={() => kiosk.sync()}
  >
    <RefreshCw class={busy ? "size-3.5 animate-spin" : "size-3.5"} />
    {kiosk.processing ? "Wird ausgeführt…" : kiosk.syncing ? "Synchronisiere…" : "Synchronisieren"}
  </button>
</footer>
