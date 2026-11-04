<script lang="ts">
  import { ArrowUpCircle, ChevronRight } from "@lucide/svelte";
  import { Button, EmptyState, PageHeader, SectionHeading } from "$lib/ui";
  import ProductCollection from "$lib/components/ProductCollection.svelte";
  import { kiosk } from "$lib/kiosk/store.svelte";
  import { selectProducts, type Filter } from "$lib/kiosk/product";

  type Props = {
    onopen: (productId: string) => void;
    onshowall: (filter: Filter) => void;
  };
  let { onopen, onshowall }: Props = $props();

  /** Tiles per home section — the "Alle anzeigen" link leads to the rest. */
  const PREVIEW_COUNT = 8;

  const available = $derived(selectProducts(kiosk.products, "available", ""));
  const installed = $derived(selectProducts(kiosk.products, "installed", ""));
  const title = $derived(kiosk.branding.companyName || "Software Center");
</script>

<PageHeader {title}>
  Apps für diesen Computer suchen, installieren und aktuell halten.
</PageHeader>

{#if kiosk.updatable.length}
  <section class="mb-8">
    <SectionHeading title="Updates verfügbar">
      {#snippet actions()}
        <Button size="sm" disabled={kiosk.processing} onclick={() => kiosk.updateAll()}>
          <ArrowUpCircle class="size-4" />
          Alle aktualisieren
        </Button>
      {/snippet}
    </SectionHeading>
    <ProductCollection products={kiosk.updatable} {onopen} />
  </section>
{/if}

{#snippet section(heading: string, filter: Filter, products: typeof available)}
  {#if products.length}
    <section class="mb-8">
      <SectionHeading title={heading}>
        {#snippet actions()}
          {#if products.length > PREVIEW_COUNT}
            <Button variant="link" size="sm" onclick={() => onshowall(filter)}>
              Alle anzeigen ({products.length})
              <ChevronRight class="size-4" />
            </Button>
          {/if}
        {/snippet}
      </SectionHeading>
      <ProductCollection products={products.slice(0, PREVIEW_COUNT)} {onopen} />
    </section>
  {/if}
{/snippet}

{@render section("Verfügbare Apps", "available", available)}
{@render section("Installierte Apps", "installed", installed)}

{#if kiosk.products.length === 0}
  <EmptyState>Für diesen Computer ist noch keine Software freigegeben.</EmptyState>
{/if}
