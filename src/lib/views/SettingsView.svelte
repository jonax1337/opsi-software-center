<script lang="ts">
  import { onMount } from "svelte";
  import { ArrowSync, Power, Spinner } from "$lib/ui/icons";
  import { getVersion } from "@tauri-apps/api/app";
  import { Button, Card, CardContent, DataField, PageHeader, SectionHeading, toast } from "$lib/ui";
  import { isTauri } from "$lib/kiosk/bridge";
  import { CONNECTION_LABEL, kiosk, type Connection } from "$lib/kiosk/store.svelte";
  import { autostartEnabled, setAutostart } from "$lib/system/desktop";
  import { formatTime } from "$lib/format";

  let appVersion = $state<string | null>(null);
  let autostart = $state(false);
  let autostartBusy = $state(false);

  onMount(() => {
    if (!isTauri()) return;
    getVersion()
      .then((v) => (appVersion = v))
      .catch(() => {
        // version row is simply hidden
      });
    void autostartEnabled().then((on) => (autostart = on));
  });

  /** setAutostart reports the state actually in effect, so the toast never lies. */
  async function toggleAutostart() {
    autostartBusy = true;
    try {
      autostart = await setAutostart(!autostart);
      toast.success(
        autostart ? "Autostart aktiviert" : "Autostart deaktiviert",
        autostart
          ? "Das Software Portal startet künftig automatisch im Infobereich."
          : "Das Software Portal startet nicht mehr automatisch.",
      );
    } finally {
      autostartBusy = false;
    }
  }

  const busy = $derived(kiosk.syncing || kiosk.processing);
  const CONNECTION_DOT: Record<Connection, string> = {
    preview: "bg-muted-foreground",
    demo: "bg-amber-500",
    connected: "bg-success",
    offline: "bg-destructive",
  };
</script>

<PageHeader title="Einstellungen">Synchronisierung und Verhalten des Software Portals.</PageHeader>

<SectionHeading title="Gerät synchronisieren" />
<Card class="card-inset mb-6">
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
      <Button disabled={busy || kiosk.loading || !isTauri()} onclick={() => kiosk.sync()}>
        <ArrowSync class={busy ? "size-4 animate-spin" : "size-4"} />
        {kiosk.processing ? "Wird ausgeführt…" : kiosk.syncing ? "Synchronisiere…" : "Synchronisieren"}
      </Button>
    </div>
  </CardContent>
</Card>

<SectionHeading title="Start" />
<Card class="card-inset mb-6">
  <CardContent>
    <div class="flex items-center justify-between gap-4">
      <div class="min-w-0">
        <p class="text-sm font-medium">Automatisch starten</p>
        <p class="text-sm text-muted-foreground">
          Beim Anmelden im Hintergrund (Infobereich) starten und auf Updates prüfen.
        </p>
      </div>
      <Button
        variant={autostart ? "default" : "outline"}
        size="sm"
        disabled={!isTauri() || autostartBusy}
        onclick={toggleAutostart}
      >
        {#if autostartBusy}<Spinner class="size-4 animate-spin" />{:else}<Power class="size-4" />{/if}
        {autostart ? "Aktiviert" : "Deaktiviert"}
      </Button>
    </div>
  </CardContent>
</Card>

<SectionHeading title="Verbindung" />
<Card class="card-inset">
  <CardContent>
    <div class="mb-5 flex items-center gap-2 text-sm">
      <span class="size-2 rounded-full {CONNECTION_DOT[kiosk.connection]}"></span>
      <span class="font-medium">{CONNECTION_LABEL[kiosk.connection]}</span>
    </div>
    <div class="grid grid-cols-2 gap-x-6 gap-y-5 sm:grid-cols-3">
      {#if kiosk.status?.clientId}
        <DataField label="opsi-Client-ID" value={kiosk.status.clientId} />
      {/if}
      {#if kiosk.status?.depotId}
        <DataField label="Depot" value={kiosk.status.depotId} />
      {/if}
      {#if kiosk.status?.connected && kiosk.status.version}
        <DataField label="opsi-Client-Agent" value={kiosk.status.version} />
      {/if}
      {#if kiosk.connected}
        <DataField label="Apps insgesamt" value={kiosk.products.length} />
      {/if}
      {#if appVersion}
        <DataField label="Software Portal" value={`v${appVersion}`} />
      {/if}
    </div>
  </CardContent>
</Card>
