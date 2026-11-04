<script lang="ts">
  import { onMount } from "svelte";
  import { Globe, Mail, Phone } from "@lucide/svelte";
  import { Card, CardContent, DataField, PageHeader, SectionHeading, toast } from "$lib/ui";
  import SupportTools from "$lib/components/SupportTools.svelte";
  import {
    getSystemInfo,
    isTauri,
    openSupportContact,
    type SupportChannel,
    type SystemInfo,
  } from "$lib/kiosk/bridge";
  import { kiosk } from "$lib/kiosk/store.svelte";
  import {
    describePasswordExpiry,
    EM_DASH,
    formatDate,
    formatDateTime,
    formatUptime,
    presentOrUndefined as present,
  } from "$lib/format";

  let sys = $state<SystemInfo | null>(null);
  let sysLoading = $state(isTauri());

  onMount(() => {
    if (!isTauri()) return;
    void kiosk.loadBranding();
    getSystemInfo()
      .then((info) => (sys = info))
      .catch(() => {
        // no inventory is shown as "nicht verfügbar" below
      })
      .finally(() => (sysLoading = false));
  });

  const branding = $derived(kiosk.branding);
  const contacts = $derived(
    [
      { channel: "phone" as const, icon: Phone, label: "Telefon", value: branding.supportPhone },
      { channel: "email" as const, icon: Mail, label: "E-Mail", value: branding.supportEmail },
      { channel: "website" as const, icon: Globe, label: "Website", value: branding.supportUrl },
    ].filter((c) => c.value.trim()),
  );

  async function contact(channel: SupportChannel) {
    try {
      await openSupportContact(channel);
    } catch (err) {
      toast.error("Kontakt konnte nicht geöffnet werden", String(err));
    }
  }

  const now = new Date();
  const uptime = $derived(sys?.pc?.lastBoot ? formatUptime(sys.pc.lastBoot, now) : undefined);
  const password = $derived(sys?.user ? describePasswordExpiry(sys.user.passwordExpires, now) : null);
  const PASSWORD_TONE_CLASS = {
    bad: "text-destructive font-medium",
    warn: "text-amber-600 dark:text-amber-400 font-medium",
    ok: "",
    muted: "",
  } as const;
  const lastUpdate = $derived.by(() => {
    const d = sys?.diag;
    if (!d?.lastUpdateId) return EM_DASH;
    return d.lastUpdate ? `${d.lastUpdateId} · ${formatDate(d.lastUpdate)}` : d.lastUpdateId;
  });
</script>

<PageHeader title="Hilfe & Support">
  Kontakt zum IT-Support und Informationen zu diesem Computer.
</PageHeader>

<SectionHeading title="IT-Support kontaktieren" />
<Card class="card-inset mb-6">
  <CardContent>
    <p class="mb-4 text-sm font-medium">{branding.companyName || "IT-Abteilung"}</p>
    {#if contacts.length}
      <div class="grid gap-2 sm:grid-cols-3">
        {#each contacts as c (c.channel)}
          <button
            type="button"
            onclick={() => contact(c.channel)}
            class="flex min-w-0 items-center gap-3 rounded-md border border-hairline bg-surface-2 px-3 py-2.5 text-left transition-colors hover:bg-surface-3 cursor-pointer outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            <c.icon class="size-4 shrink-0 text-primary" />
            <span class="min-w-0">
              <span class="block text-xs text-muted-foreground">{c.label}</span>
              <span class="block truncate text-sm">{c.value}</span>
            </span>
          </button>
        {/each}
      </div>
    {:else}
      <p class="text-sm text-muted-foreground">
        Wenden Sie sich bei Fragen an Ihre IT-Abteilung. Kontaktdaten kann die IT im opsi-Paket
        des Software Centers hinterlegen.
      </p>
    {/if}
  </CardContent>
</Card>

<SectionHeading title="Computer" />
<Card class="card-inset mb-6">
  <CardContent>
    {#if sys?.pc}
      {@const pc = sys.pc}
      <div class="grid grid-cols-2 gap-x-6 gap-y-5 sm:grid-cols-3">
        <DataField label="Computername" value={pc.hostname} mono />
        <DataField label="Domäne" value={present(pc.domain)} />
        <DataField label="Modell" value={present(`${pc.manufacturer} ${pc.model}`)} />
        <DataField label="Seriennummer" value={present(pc.serial)} mono />
        <DataField label="Windows" value={pc.os} />
        <DataField label="Build" value={pc.build} mono />
        <DataField label="Prozessor" value={present(pc.cpu)} />
        <DataField label="Arbeitsspeicher" value={`${pc.ramGb} GB`} />
        <DataField label="Speicher (C:)" value={`${pc.diskFreeGb} GB frei / ${pc.diskTotalGb} GB`} />
        <DataField label="IP-Adresse" value={pc.ip ?? EM_DASH} mono />
        <DataField label="Letzter Neustart" value={formatDateTime(pc.lastBoot)} />
        <DataField label="Betriebszeit" value={uptime} />
      </div>
    {:else}
      <p class="text-sm text-muted-foreground">
        {sysLoading ? "Geräteinformationen werden geladen…" : "Keine Geräteinformationen verfügbar."}
      </p>
    {/if}
  </CardContent>
</Card>

<SectionHeading title="Mein Konto" />
<Card class="card-inset mb-6">
  <CardContent>
    {#if sys?.user}
      {@const u = sys.user}
      <div class="grid grid-cols-2 gap-x-6 gap-y-5 sm:grid-cols-3">
        <DataField label="Name" value={present(u.displayName)} />
        <DataField label="Benutzer" value={u.username} mono />
        <DataField label="E-Mail" value={present(u.mail)} />
        <DataField label="Abteilung" value={present(u.department)} />
        <DataField label="Büro" value={present(u.office)} />
        <DataField label="Position" value={present(u.title)} />
        {#if password}
          <DataField label="Passwort läuft ab">
            <span class={PASSWORD_TONE_CLASS[password.tone]}>{password.text}</span>
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

<SectionHeading title="Wartung" />
<Card class="card-inset">
  <CardContent>
    <div class="space-y-5">
      {#if sys?.diag}
        {@const d = sys.diag}
        <div class="grid grid-cols-2 gap-x-6 gap-y-5 sm:grid-cols-3">
          <DataField label="Anmeldeserver (DC)" value={present(d.logonServer)} mono />
          <DataField
            label="Letzte Richtlinienaktualisierung"
            value={d.lastGpo ? formatDateTime(d.lastGpo) : EM_DASH}
          />
          <DataField label="Letztes Update" value={lastUpdate} />
        </div>
      {/if}
      <SupportTools />
    </div>
  </CardContent>
</Card>
