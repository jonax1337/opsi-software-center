<script lang="ts">
  import { onMount } from "svelte";
  import { Laptop } from "$lib/ui/icons";
  import { Card, CardContent, DataField, InfoBanner, PageHeader, SectionHeading } from "$lib/ui";
  import AdminActions from "$lib/components/AdminActions.svelte";
  import SupportTools from "$lib/components/SupportTools.svelte";
  import SyncCard from "$lib/components/SyncCard.svelte";
  import { kiosk } from "$lib/kiosk/store.svelte";
  import { systemInfo } from "$lib/system/systemInfo.svelte";
  import {
    EM_DASH,
    formatDate,
    formatDateTime,
    formatUptime,
    presentOrUndefined as present,
  } from "$lib/format";

  /** Company Portal "Geräte": this computer, its details and self-help tools. */
  onMount(() => systemInfo.load());

  const pc = $derived(systemInfo.info?.pc);
  const diag = $derived(systemInfo.info?.diag);
  const uptime = $derived(pc?.lastBoot ? formatUptime(pc.lastBoot, new Date()) : undefined);
  const lastUpdate = $derived.by(() => {
    if (!diag?.lastUpdateId) return EM_DASH;
    return diag.lastUpdate ? `${diag.lastUpdateId} · ${formatDate(diag.lastUpdate)}` : diag.lastUpdateId;
  });
</script>

<PageHeader title="Geräte" />

<div class="mb-8 flex items-center gap-5">
  <div class="grid size-20 shrink-0 place-items-center rounded-xl bg-surface-3">
    <Laptop class="size-10 text-primary" />
  </div>
  <div class="min-w-0">
    <h2 class="truncate text-[28px] font-semibold leading-9">{pc?.hostname ?? "Dieser Computer"}</h2>
    <p class="text-sm text-muted-foreground">
      {pc ? (present(`${pc.manufacturer} ${pc.model}`) ?? pc.os) : "Dieses Gerät"}
    </p>
  </div>
</div>

<SyncCard class="mb-8" />

<SectionHeading title="Gerätedetails" />
<Card class="card-inset mb-8">
  <CardContent>
    {#if pc}
      <div class="grid grid-cols-2 gap-x-6 gap-y-5 @xl:grid-cols-3">
        <DataField label="Computername" value={pc.hostname} />
        <DataField label="Domäne" value={present(pc.domain)} />
        <DataField label="Seriennummer" value={present(pc.serial)} />
        <DataField label="Windows" value={pc.os} />
        <DataField label="Build" value={pc.build} />
        <DataField label="Prozessor" value={present(pc.cpu)} />
        <DataField label="Arbeitsspeicher" value={`${pc.ramGb} GB`} />
        <DataField label="Speicher (C:)" value={`${pc.diskFreeGb} GB frei / ${pc.diskTotalGb} GB`} />
        <DataField label="IP-Adresse" value={pc.ip ?? EM_DASH} />
        <DataField label="Letzter Neustart" value={formatDateTime(pc.lastBoot)} />
        <DataField label="Betriebszeit" value={uptime} />
      </div>
    {:else}
      <p class="text-sm text-muted-foreground">
        {systemInfo.loading
          ? "Geräteinformationen werden geladen…"
          : "Keine Geräteinformationen verfügbar."}
      </p>
    {/if}
  </CardContent>
</Card>

<SectionHeading title="Wartung" />
<Card class="card-inset">
  <CardContent>
    <div class="space-y-5">
      {#if kiosk.rebootPending}
        <InfoBanner tone="warning">
          Ein Neustart steht aus, um Aktualisierungen abzuschließen. Starten Sie den Computer neu, sobald
          es passt.
        </InfoBanner>
      {/if}
      {#if diag}
        <div class="grid grid-cols-2 gap-x-6 gap-y-5 @xl:grid-cols-3">
          <DataField label="Anmeldeserver (DC)" value={present(diag.logonServer)} />
          <DataField
            label="Letzte Richtlinienaktualisierung"
            value={diag.lastGpo ? formatDateTime(diag.lastGpo) : EM_DASH}
          />
          <DataField label="Letztes Update" value={lastUpdate} />
        </div>
      {/if}
      <SupportTools />
    </div>
  </CardContent>
</Card>

<AdminActions />
