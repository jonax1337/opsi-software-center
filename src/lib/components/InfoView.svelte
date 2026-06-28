<script lang="ts">
  import { FileText, Globe, Loader2, Power, RefreshCw, RotateCw } from "@lucide/svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import {
    Button,
    Card,
    CardContent,
    DataField,
    InfoBanner,
    PageHeader,
    SectionHeading,
    toast,
  } from "$lib/ui";
  import {
    deviceFlushDns,
    deviceGpresultReport,
    deviceGpupdate,
    getSystemInfo,
    isTauri,
    type SystemInfo,
  } from "$lib/kiosk/bridge";
  import { kiosk } from "$lib/kiosk/store.svelte";
  import { autostartEnabled, setAutostart } from "$lib/system/desktop";

  let sys = $state<SystemInfo | null>(null);
  let sysLoading = $state(false);
  let appVersion = $state<string | null>(null);
  let autostart = $state(false);
  let autostartBusy = $state(false);

  $effect(() => {
    if (!isTauri()) return;
    sysLoading = true;
    getSystemInfo()
      .then((s) => (sys = s))
      .catch(() => {})
      .finally(() => (sysLoading = false));
    getVersion()
      .then((v) => (appVersion = v))
      .catch(() => {});
    autostartEnabled()
      .then((on) => (autostart = on))
      .catch(() => {});
  });

  async function toggleAutostart() {
    autostartBusy = true;
    try {
      autostart = await setAutostart(!autostart);
      toast.success(
        autostart ? "Autostart aktiviert" : "Autostart deaktiviert",
        autostart
          ? "Das Software Center startet künftig automatisch im Infobereich."
          : "Das Software Center startet nicht mehr automatisch.",
      );
    } catch {
      toast.error("Autostart konnte nicht geändert werden");
    } finally {
      autostartBusy = false;
    }
  }

  const conn = $derived.by(() => {
    if (!isTauri()) return { dot: "bg-muted-foreground", label: "Browser-Vorschau" };
    if (kiosk.status?.mock) return { dot: "bg-amber-500", label: "Demo-Modus" };
    if (kiosk.status?.connected) return { dot: "bg-success", label: "Verbunden" };
    return { dot: "bg-destructive", label: "Nicht verbunden" };
  });
  const busy = $derived(kiosk.syncing || kiosk.processing);
  const lastSyncLabel = $derived(
    kiosk.lastSync
      ? kiosk.lastSync.toLocaleTimeString("de-DE", { hour: "2-digit", minute: "2-digit" })
      : null,
  );

  // Empty AD strings → undefined so DataField renders an em-dash.
  const v = (s?: string | null) => (s && s.trim() ? s.trim() : undefined);

  function fmtDateTime(iso: string): string {
    const d = new Date(iso);
    return Number.isNaN(d.getTime()) ? "—" : d.toLocaleString("de-DE");
  }

  const uptime = $derived.by(() => {
    if (!sys?.pc?.lastBoot) return undefined;
    const ms = Date.now() - new Date(sys.pc.lastBoot).getTime();
    if (!(ms > 0)) return undefined;
    const h = Math.floor(ms / 3_600_000);
    const d = Math.floor(h / 24);
    return d > 0 ? `${d} d ${h % 24} h` : `${h} h`;
  });

  // Password expiry → human text + tone for colouring.
  const pw = $derived.by(() => {
    const u = sys?.user;
    if (!u) return null;
    if (!u.passwordExpires) return { text: "Läuft nicht ab", tone: "muted" as const };
    const exp = new Date(u.passwordExpires);
    if (Number.isNaN(exp.getTime())) return null;
    const days = Math.ceil((exp.getTime() - Date.now()) / 86_400_000);
    const date = exp.toLocaleDateString("de-DE");
    if (days < 0) return { text: `Abgelaufen (${date})`, tone: "bad" as const };
    if (days === 0) return { text: "Läuft heute ab", tone: "warn" as const };
    const t = `In ${days} ${days === 1 ? "Tag" : "Tagen"} (${date})`;
    return { text: t, tone: (days <= 14 ? "warn" : "ok") as "warn" | "ok" };
  });
  const pwClass = $derived(
    pw?.tone === "bad"
      ? "text-destructive font-medium"
      : pw?.tone === "warn"
        ? "text-amber-600 dark:text-amber-400 font-medium"
        : "",
  );

  let gpBusy = $state(false);
  let dnsBusy = $state(false);
  let gprBusy = $state(false);

  async function runGpupdate() {
    gpBusy = true;
    try {
      const r = await deviceGpupdate(true);
      if (r.computer && r.user) {
        toast.success("Richtlinien aktualisiert", "Computer und Benutzer erfolgreich.");
      } else {
        toast.warning(
          "Teilweise aktualisiert",
          `Computer: ${r.computer ? "OK" : "Fehler"}, Benutzer: ${r.user ? "OK" : "Fehler"}`,
        );
      }
    } catch {
      toast.error("gpupdate fehlgeschlagen");
    } finally {
      gpBusy = false;
    }
  }

  async function flushDns() {
    dnsBusy = true;
    try {
      if (await deviceFlushDns()) toast.success("DNS-Cache geleert");
      else toast.error("DNS-Cache leeren fehlgeschlagen");
    } catch {
      toast.error("Fehlgeschlagen");
    } finally {
      dnsBusy = false;
    }
  }

  async function gpReport() {
    gprBusy = true;
    try {
      await deviceGpresultReport();
      toast.success("Richtlinienbericht erstellt", "Wird im Browser geöffnet.");
    } catch {
      toast.error("Bericht fehlgeschlagen");
    } finally {
      gprBusy = false;
    }
  }
</script>

<PageHeader title="Info" description="Gerät, Konto und Status dieser Installation.">
  {#snippet actions()}
    <Button
      variant="outline"
      size="sm"
      disabled={busy || kiosk.loading || !isTauri()}
      onclick={() => kiosk.sync()}
    >
      <RefreshCw class={busy ? "size-4 animate-spin" : "size-4"} />
      {kiosk.processing ? "Wird ausgeführt…" : kiosk.syncing ? "Synchronisiere…" : "Synchronisieren"}
    </Button>
  {/snippet}
</PageHeader>

<div class="mb-5 flex items-center gap-2 text-sm">
  <span class={`size-2 rounded-full ${conn.dot}`}></span>
  <span class="font-medium">{conn.label}</span>
</div>

{#if kiosk.rebootPending}
  <InfoBanner tone="warning" class="mb-6">
    Neustart ausstehend — bitte den Computer neu starten, um ausstehende Aktualisierungen
    abzuschließen.
  </InfoBanner>
{/if}

<!-- Computer -->
<SectionHeading title="Computer" />
<Card class="card-inset mb-6">
  <CardContent>
    {#if sys?.pc}
      {@const pc = sys.pc}
      <div class="grid grid-cols-2 sm:grid-cols-3 gap-x-6 gap-y-5">
        <DataField label="Computername" value={pc.hostname} mono />
        <DataField label="Domäne" value={v(pc.domain)} />
        <DataField label="Modell" value={`${pc.manufacturer} ${pc.model}`} />
        <DataField label="Seriennummer" value={v(pc.serial)} mono />
        <DataField label="Windows" value={pc.os} />
        <DataField label="Build" value={pc.build} mono />
        <DataField label="Prozessor" value={v(pc.cpu)} />
        <DataField label="Arbeitsspeicher" value={`${pc.ramGb} GB`} />
        <DataField label="Speicher (C:)" value={`${pc.diskFreeGb} GB frei / ${pc.diskTotalGb} GB`} />
        <DataField label="IP-Adresse" value={pc.ip ?? "—"} mono />
        <DataField label="Letzter Neustart" value={fmtDateTime(pc.lastBoot)} />
        <DataField label="Betriebszeit" value={uptime} />
      </div>
    {:else}
      <p class="text-sm text-muted-foreground">
        {sysLoading ? "Geräteinformationen werden geladen…" : "Keine Geräteinformationen verfügbar."}
      </p>
    {/if}
  </CardContent>
</Card>

<!-- Konto -->
<SectionHeading title="Mein Konto" />
<Card class="card-inset mb-6">
  <CardContent>
    {#if sys?.user}
      {@const u = sys.user}
      <div class="grid grid-cols-2 sm:grid-cols-3 gap-x-6 gap-y-5">
        <DataField label="Name" value={v(u.displayName)} />
        <DataField label="Benutzer" value={u.username} mono />
        <DataField label="E-Mail" value={v(u.mail)} />
        <DataField label="Abteilung" value={v(u.department)} />
        <DataField label="Büro" value={v(u.office)} />
        <DataField label="Position" value={v(u.title)} />
        {#if pw}
          <DataField label="Passwort läuft ab">
            <span class={pwClass}>{pw.text}</span>
          </DataField>
        {/if}
      </div>
    {:else}
      <p class="text-sm text-muted-foreground">
        {sysLoading
          ? "Kontoinformationen werden geladen…"
          : "Keine Verzeichnisverbindung — Kontodaten nicht verfügbar."}
      </p>
    {/if}
  </CardContent>
</Card>

<!-- Verwaltung (opsi) -->
<SectionHeading title="Verwaltung" />
<Card class="card-inset mb-6">
  <CardContent>
    <div class="grid grid-cols-2 sm:grid-cols-3 gap-x-6 gap-y-5">
      {#if kiosk.status?.clientId}
        <DataField label="opsi-Client-ID" value={kiosk.status.clientId} mono />
      {/if}
      {#if kiosk.status?.depotId}
        <DataField label="Depot" value={kiosk.status.depotId} mono />
      {/if}
      {#if kiosk.status?.version && kiosk.status.version !== "mock"}
        <DataField label="opsi-Client-Agent" value={kiosk.status.version} mono />
      {/if}
      {#if !kiosk.loading}
        <DataField label="Apps insgesamt" value={kiosk.products.length} />
      {/if}
      {#if lastSyncLabel}
        <DataField label="Zuletzt synchronisiert" value={`${lastSyncLabel} Uhr`} />
      {/if}
      {#if appVersion}
        <DataField label="Software Center" value={`v${appVersion}`} mono />
      {/if}
    </div>
  </CardContent>
</Card>

<!-- Einstellungen -->
<SectionHeading title="Einstellungen" />
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
        {#if autostartBusy}<Loader2 class="size-4 animate-spin" />{:else}<Power class="size-4" />{/if}
        {autostart ? "Aktiviert" : "Deaktiviert"}
      </Button>
    </div>
  </CardContent>
</Card>

<!-- Wartung -->
<SectionHeading title="Wartung" />
<Card class="card-inset">
  <CardContent>
    <div class="space-y-5">
      {#if sys?.diag}
        {@const d = sys.diag}
        <div class="grid grid-cols-2 sm:grid-cols-3 gap-x-6 gap-y-5">
          <DataField label="Anmeldeserver (DC)" value={v(d.logonServer)} mono />
          <DataField
            label="Letzte Richtlinienaktualisierung"
            value={d.lastGpo ? fmtDateTime(d.lastGpo) : "—"}
          />
          <DataField
            label="Letztes Update"
            value={d.lastUpdateId
              ? `${d.lastUpdateId}${d.lastUpdate ? ` · ${new Date(d.lastUpdate).toLocaleDateString("de-DE")}` : ""}`
              : "—"}
          />
        </div>
      {/if}

      <div class="flex flex-wrap gap-2">
        <Button variant="outline" size="sm" disabled={!isTauri() || gpBusy} onclick={runGpupdate}>
          {#if gpBusy}<Loader2 class="size-4 animate-spin" />{:else}<RotateCw class="size-4" />{/if}
          Richtlinien aktualisieren
        </Button>
        <Button variant="outline" size="sm" disabled={!isTauri() || dnsBusy} onclick={flushDns}>
          {#if dnsBusy}<Loader2 class="size-4 animate-spin" />{:else}<Globe class="size-4" />{/if}
          DNS-Cache leeren
        </Button>
        <Button variant="outline" size="sm" disabled={!isTauri() || gprBusy} onclick={gpReport}>
          {#if gprBusy}<Loader2 class="size-4 animate-spin" />{:else}<FileText class="size-4" />{/if}
          Richtlinienbericht
        </Button>
      </div>
    </div>
  </CardContent>
</Card>
