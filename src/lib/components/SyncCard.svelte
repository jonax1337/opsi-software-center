<script lang="ts">
  import { ArrowSync } from "$lib/ui/icons";
  import { Button, Card, CardContent, SectionHeading } from "$lib/ui";
  import { isTauri } from "$lib/kiosk/bridge";
  import { kiosk } from "$lib/kiosk/store.svelte";
  import { formatTime } from "$lib/format";

  /** "Gerät synchronisieren" — shown on the device page and in settings. */
  let { class: className = "" }: { class?: string } = $props();

  const busy = $derived(kiosk.syncing || kiosk.processing);
</script>

<SectionHeading title="Gerät synchronisieren" />
<Card class="card-inset {className}">
  <CardContent>
    <div class="flex flex-wrap items-center justify-between gap-4">
      <div class="min-w-0">
        <p class="text-sm">
          Holt neue Zuweisungen vom Server, führt ausstehende Installationen aus und aktualisiert
          die Gruppenrichtlinien.
        </p>
        <p class="mt-1 text-sm text-muted-foreground">
          {kiosk.lastSync
            ? `Zuletzt synchronisiert: ${formatTime(kiosk.lastSync)} Uhr`
            : "Noch nicht synchronisiert."}
        </p>
      </div>
      <Button size="lg" disabled={busy || kiosk.loading || !isTauri()} onclick={() => kiosk.sync()}>
        <ArrowSync class={busy ? "size-4 animate-spin" : "size-4"} />
        {kiosk.processing ? "Wird ausgeführt…" : kiosk.syncing ? "Synchronisiere…" : "Synchronisieren"}
      </Button>
    </div>
  </CardContent>
</Card>
