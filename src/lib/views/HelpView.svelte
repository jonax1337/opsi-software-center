<script lang="ts">
  import { onMount } from "svelte";
  import { Call, Globe, Mail } from "$lib/ui/icons";
  import { Card, CardContent, DataField, PageHeader, SectionHeading, toast } from "$lib/ui";
  import { isTauri, openSupportContact, type SupportChannel } from "$lib/kiosk/bridge";
  import { kiosk } from "$lib/kiosk/store.svelte";
  import { systemInfo } from "$lib/system/systemInfo.svelte";
  import { describePasswordExpiry, presentOrUndefined as present } from "$lib/format";

  onMount(() => {
    if (!isTauri()) return;
    void kiosk.loadBranding();
    systemInfo.load();
  });

  const sys = $derived(systemInfo.info);

  const branding = $derived(kiosk.branding);
  const contacts = $derived(
    [
      { channel: "phone" as const, icon: Call, label: "Telefon", value: branding.supportPhone },
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

  const password = $derived(
    sys?.user ? describePasswordExpiry(sys.user.passwordExpires, new Date()) : null,
  );
  const PASSWORD_TONE_CLASS = {
    bad: "text-destructive font-medium",
    warn: "text-amber-600 dark:text-amber-400 font-medium",
    ok: "",
    muted: "",
  } as const;
</script>

<PageHeader title="Hilfe und Support">
  Kontakt zum IT-Support und Informationen zu Ihrem Konto.
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

<SectionHeading title="Mein Konto" />
<Card class="card-inset">
  <CardContent>
    {#if sys?.user}
      {@const u = sys.user}
      <div class="grid grid-cols-2 gap-x-6 gap-y-5 sm:grid-cols-3">
        <DataField label="Name" value={present(u.displayName)} />
        <DataField label="Benutzer" value={u.username} />
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
        {systemInfo.loading
          ? "Kontoinformationen werden geladen…"
          : "Keine Verzeichnisverbindung — Kontodaten nicht verfügbar."}
      </p>
    {/if}
  </CardContent>
</Card>
