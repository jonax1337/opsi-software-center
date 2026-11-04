<script lang="ts">
  import {
    ArrowCircleUp,
    ArrowCounterclockwise,
    ArrowSync,
    CheckmarkCircle,
    Delete,
    Info,
  } from "$lib/ui/icons";
  import { Button, InfoBanner, OverflowMenu, PageHeader, SectionHeading } from "$lib/ui";
  import type { MenuItem } from "$lib/ui/OverflowMenu.svelte";
  import ProductTable from "$lib/components/ProductTable.svelte";
  import { kiosk } from "$lib/kiosk/store.svelte";
  import { displayName, isPending, sortProducts } from "$lib/kiosk/product";
  import type { KioskProduct } from "$lib/kiosk/types";

  /** Company Portal "Downloads und Updates": queue, available updates and
   *  the installed apps, each as a table. */
  let { onopen }: { onopen: (productId: string) => void } = $props();

  const busy = $derived(kiosk.syncing || kiosk.processing);
  const installed = $derived(
    sortProducts(
      kiosk.installed.filter((p) => !isPending(p)),
      "name-asc",
    ),
  );
  const resultCount = $derived(kiosk.pending.length + kiosk.updatesToOffer.length + installed.length);

  function installedActions(p: KioskProduct): MenuItem[] {
    const id = p.productId;
    return [
      { label: "Details anzeigen", icon: Info, onselect: () => onopen(id) },
      {
        label: "Neu installieren",
        icon: ArrowCounterclockwise,
        disabled: !p.hasSetup || kiosk.processing,
        onselect: () => void kiosk.runAction(id, "setup"),
      },
      {
        label: "Deinstallieren",
        icon: Delete,
        danger: true,
        disabled: !p.hasUninstall || kiosk.processing,
        onselect: () => void kiosk.runAction(id, "uninstall"),
      },
    ];
  }
</script>

<PageHeader title="Downloads und Updates">
  {resultCount === 1 ? "1 Ergebnis" : `${resultCount} Ergebnisse`}
  {#snippet actions()}
    <Button variant="outline" size="sm" disabled={busy} onclick={() => kiosk.sync()}>
      <ArrowSync class={busy ? "size-4 animate-spin" : "size-4"} />
      {kiosk.syncing ? "Wird gesucht…" : "Nach Updates suchen"}
    </Button>
  {/snippet}
</PageHeader>

{#if kiosk.rebootPending}
  <InfoBanner tone="warning" class="mb-6">
    Neustart ausstehend — bitte den Computer neu starten, um Aktualisierungen abzuschließen.
  </InfoBanner>
{/if}

{#if kiosk.pending.length}
  <section class="mb-8">
    <SectionHeading title={`In der Warteschlange (${kiosk.pending.length})`} />
    <ProductTable products={kiosk.pending} {onopen} />
  </section>
{/if}

{#if kiosk.updatesToOffer.length}
  <section class="mb-8">
    <SectionHeading title={`Updates verfügbar (${kiosk.updatesToOffer.length})`}>
      {#snippet actions()}
        <Button size="sm" disabled={kiosk.processing} onclick={() => kiosk.updateAll()}>
          <ArrowCircleUp class="size-4" />
          Alle aktualisieren
        </Button>
      {/snippet}
    </SectionHeading>
    <ProductTable products={kiosk.updatesToOffer} {onopen}>
      {#snippet actions(p)}
        <Button
          size="sm"
          variant="outline"
          aria-label={`${displayName(p)} aktualisieren`}
          disabled={kiosk.processing || kiosk.busy.has(p.productId)}
          onclick={() => kiosk.runAction(p.productId, "setup")}
        >
          <ArrowCircleUp class="size-4" />
        </Button>
      {/snippet}
    </ProductTable>
  </section>
{:else if !kiosk.pending.length}
  <InfoBanner tone="success" class="mb-8">
    <span class="inline-flex items-center gap-2">
      <CheckmarkCircle class="size-4" /> Alle Apps sind auf dem neuesten Stand.
    </span>
  </InfoBanner>
{/if}

{#if installed.length}
  <section>
    <SectionHeading title={`Installierte Apps (${installed.length})`} />
    <ProductTable products={installed} {onopen}>
      {#snippet actions(p)}
        <OverflowMenu label={`Weitere Aktionen für ${displayName(p)}`} items={installedActions(p)} />
      {/snippet}
    </ProductTable>
  </section>
{/if}
