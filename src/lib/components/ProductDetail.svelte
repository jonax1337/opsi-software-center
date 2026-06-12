<script lang="ts">
  import {
    ArrowLeft,
    ArrowUpCircle,
    Download,
    Loader2,
    Package,
    Trash2,
    X,
  } from "@lucide/svelte";
  import { Badge, Button, Card, CardContent, DataField, InfoBanner, SectionHeading } from "$lib/ui";
  import { kiosk } from "$lib/kiosk/store.svelte";
  import type { KioskProduct } from "$lib/kiosk/types";

  type Props = { product: KioskProduct; onback: () => void };
  let { product, onback }: Props = $props();

  const installed = $derived(product.installationStatus === "installed");
  const pending = $derived(product.actionRequest !== "" && product.actionRequest !== "none");
  const busy = $derived(kiosk.busy.has(product.productId));
  const running = $derived(pending && kiosk.processing);
  const icon = $derived(kiosk.icons[product.productId]);

  const hasBadges = $derived(installed || pending || product.actionResult === "failed");

  const requirements = $derived(
    (product.requirements ?? []).map((r) => r.requiredProductId).filter(Boolean),
  );
</script>

<button
  type="button"
  onclick={onback}
  class="inline-flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors mb-6 cursor-pointer"
>
  <ArrowLeft class="size-4" />
  Alle Apps
</button>

<div class="flex flex-wrap items-start gap-5 mb-8">
  <div
    class="grid place-items-center size-20 rounded-xl bg-surface-3 border border-hairline overflow-hidden shrink-0"
  >
    {#if icon}
      <img src={icon} alt="" class="size-13 object-contain" />
    {:else}
      <Package class="size-9 text-muted-foreground" />
    {/if}
  </div>

  <div class="flex-1 min-w-0 self-center">
    <h1 class="text-2xl font-semibold tracking-tight truncate leading-tight">
      {product.productName || product.productId}
    </h1>
    <div class="mt-1 text-sm text-muted-foreground font-mono tabular-nums">
      {installed ? product.installedVerStr : product.versionStr}
    </div>
    {#if hasBadges}
      <div class="mt-2.5 flex items-center gap-2 flex-wrap">
        {#if installed}
          <Badge variant="success">Installiert</Badge>
          {#if product.updatePossible}
            <Badge variant="warning">Update auf {product.versionStr}</Badge>
          {/if}
        {/if}
        {#if product.actionResult === "failed"}
          <Badge variant="destructive">Letzte Aktion fehlgeschlagen</Badge>
        {/if}
        {#if running}
          <Badge variant="secondary">
            <Loader2 class="size-2.5 animate-spin" />
            {product.actionRequest === "uninstall" ? "Wird deinstalliert…" : "Wird installiert…"}
          </Badge>
        {:else if pending}
          <Badge variant="secondary">Aktion ausstehend</Badge>
        {/if}
      </div>
    {/if}
  </div>

  <div class="flex items-center gap-2 shrink-0 self-center">
    {#if busy}
      <Button disabled>
        <Loader2 class="size-4 animate-spin" />
        Bitte warten…
      </Button>
    {:else if running}
      <Button disabled>
        <Loader2 class="size-4 animate-spin" />
        Wird ausgeführt…
      </Button>
    {:else if pending}
      <Button variant="outline" onclick={() => kiosk.cancelAction(product.productId)}>
        <X class="size-4" />
        Abbrechen
      </Button>
    {:else if installed}
      {#if product.hasUninstall}
        <Button
          variant="outline"
          class="text-destructive hover:text-destructive"
          disabled={kiosk.processing}
          onclick={() => kiosk.runAction(product.productId, "uninstall")}
        >
          <Trash2 class="size-4" />
          Deinstallieren
        </Button>
      {/if}
      {#if product.updatePossible}
        <Button
          disabled={kiosk.processing}
          onclick={() => kiosk.runAction(product.productId, "setup")}
        >
          <ArrowUpCircle class="size-4" />
          Aktualisieren
        </Button>
      {/if}
    {:else if product.hasSetup}
      <Button
        disabled={kiosk.processing}
        onclick={() => kiosk.runAction(product.productId, "setup")}
      >
        <Download class="size-4" />
        Installieren
      </Button>
    {/if}
  </div>
</div>

{#if product.advice}
  <InfoBanner tone="warning" class="mb-6">{product.advice}</InfoBanner>
{/if}

{#if product.description}
  <SectionHeading title="Beschreibung" />
  <Card class="card-inset mb-6">
    <CardContent>
      <p class="text-sm text-muted-foreground leading-relaxed whitespace-pre-line">
        {product.description}
      </p>
    </CardContent>
  </Card>
{/if}

<SectionHeading title="Details" />
<Card class="card-inset">
  <CardContent>
    <div class="grid grid-cols-2 sm:grid-cols-3 gap-x-6 gap-y-5">
      <DataField label="Version" value={product.versionStr} mono />
      <DataField label="Installierte Version" value={product.installedVerStr || "—"} mono />
      <DataField label="Produkt-ID" value={product.productId} mono />
      {#if requirements.length}
        <DataField label="Abhängigkeiten" value={requirements.join(", ")} />
      {/if}
    </div>
  </CardContent>
</Card>
