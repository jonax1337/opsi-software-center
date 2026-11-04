<script lang="ts">
  import type { Snippet } from "svelte";
  import { ArrowSync, PlugDisconnected } from "$lib/ui/icons";
  import { Button, EmptyState, InfoBanner } from "$lib/ui";
  import { kiosk } from "$lib/kiosk/store.svelte";
  import { isTauri } from "$lib/kiosk/bridge";

  /** Shows the catalog pages only once there is a catalog: loading spinner,
   *  browser-preview note or the "no connection" state with a retry. */
  let { children }: { children: Snippet } = $props();
</script>

{#if !isTauri()}
  <InfoBanner tone="info" class="mb-6">
    Browser-Vorschau — der opsi-Client-Dienst ist nur in der Desktop-App erreichbar.
  </InfoBanner>
  {@render children()}
{:else if kiosk.loading}
  <EmptyState loading>Apps werden geladen…</EmptyState>
{:else if !kiosk.connected}
  <div class="grid place-items-center py-20 text-center">
    <PlugDisconnected class="mb-3 size-8 text-muted-foreground" />
    <h2 class="text-base font-semibold">Keine Verbindung zum opsi-Client-Dienst</h2>
    <p class="mt-1 max-w-md text-sm text-muted-foreground">
      Der Dienst „opsiclientd" antwortet nicht. Kurz nach dem Anmelden startet er manchmal noch —
      das Software Portal versucht es automatisch erneut.
    </p>
    {#if kiosk.status?.error}
      <p class="mt-3 max-w-md break-all font-mono text-xs text-muted-foreground/80">
        {kiosk.status.error}
      </p>
    {/if}
    <Button class="mt-5" variant="outline" onclick={() => kiosk.retry()}>
      <ArrowSync class="size-4" />
      Erneut versuchen
    </Button>
  </div>
{:else}
  {#if kiosk.status?.mock}
    <InfoBanner tone="warning" class="mb-6">
      Demo-Modus — es werden Beispiel-Apps angezeigt, nichts wird wirklich installiert.
    </InfoBanner>
  {/if}
  {@render children()}
{/if}
