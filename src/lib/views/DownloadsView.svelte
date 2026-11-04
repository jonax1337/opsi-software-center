<script lang="ts">
  import { ArrowUpCircle, CircleCheck, RefreshCw } from "@lucide/svelte";
  import { Button, EmptyState, InfoBanner, PageHeader, SectionHeading } from "$lib/ui";
  import ProductCollection from "$lib/components/ProductCollection.svelte";
  import { kiosk } from "$lib/kiosk/store.svelte";

  let { onopen }: { onopen: (productId: string) => void } = $props();

  const busy = $derived(kiosk.syncing || kiosk.processing);
  const upToDate = $derived(!kiosk.pending.length && !kiosk.updatesToOffer.length);
</script>

<PageHeader title="Downloads und Updates">
  Laufende Installationen und verfügbare Aktualisierungen.
  {#snippet actions()}
    <Button variant="outline" size="sm" disabled={busy} onclick={() => kiosk.sync()}>
      <RefreshCw class={busy ? "size-4 animate-spin" : "size-4"} />
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
    <SectionHeading title={kiosk.processing ? "Wird ausgeführt" : "Ausstehend"} />
    <ProductCollection products={kiosk.pending} layout="list" {onopen} />
  </section>
{/if}

{#if kiosk.updatesToOffer.length}
  <section class="mb-8">
    <SectionHeading title="Updates verfügbar">
      {#snippet actions()}
        <Button size="sm" disabled={kiosk.processing} onclick={() => kiosk.updateAll()}>
          <ArrowUpCircle class="size-4" />
          {kiosk.updatesToOffer.length === 1
            ? "1 Update installieren"
            : `${kiosk.updatesToOffer.length} Updates installieren`}
        </Button>
      {/snippet}
    </SectionHeading>
    <ProductCollection products={kiosk.updatesToOffer} layout="list" {onopen} />
  </section>
{/if}

{#if upToDate}
  <EmptyState icon={CircleCheck} iconClass="text-success">
    Alle Apps sind auf dem neuesten Stand.
  </EmptyState>
{/if}
