<script lang="ts">
  import { onMount } from "svelte";
  import { ArrowUpCircle, SearchX } from "@lucide/svelte";
  import {
    Button,
    EmptyState,
    FilterChip,
    InfoBanner,
    PageHeader,
    SearchInput,
    Titlebar,
    Toaster,
  } from "$lib/ui";
  import ProductTile from "$lib/components/ProductTile.svelte";
  import ProductDetail from "$lib/components/ProductDetail.svelte";
  import StatusFooter from "$lib/components/StatusFooter.svelte";
  import { kiosk, type Filter } from "$lib/kiosk/store.svelte";
  import { isTauri } from "$lib/kiosk/bridge";

  const FILTERS: { value: Filter; label: string }[] = [
    { value: "all", label: "Alle" },
    { value: "available", label: "Verfügbar" },
    { value: "installed", label: "Installiert" },
    { value: "updates", label: "Updates" },
  ];

  let selectedId = $state<string | null>(null);

  const selected = $derived(
    selectedId ? (kiosk.products.find((p) => p.productId === selectedId) ?? null) : null,
  );

  function filterCount(f: Filter): number {
    if (f === "all") return kiosk.products.length;
    if (f === "installed") return kiosk.installed.length;
    if (f === "updates") return kiosk.updatable.length;
    return kiosk.products.length - kiosk.installed.length;
  }

  onMount(() => {
    void kiosk.init();
  });
</script>

<div class="h-full flex flex-col">
  <Titlebar title="Software Center" />

  <main class="flex-1 overflow-y-auto">
    <div class="max-w-4xl mx-auto px-8 pt-8 pb-12">
      {#if selected}
        <div class="animate-route">
          <ProductDetail product={selected} onback={() => (selectedId = null)} />
        </div>
      {:else}
        <div class="animate-route">
          <PageHeader
            title="Apps"
            description="Freigegebene Software für diesen Computer installieren, aktualisieren oder entfernen."
          >
            {#snippet actions()}
              {#if kiosk.updatable.length > 0}
                <Button
                  size="sm"
                  disabled={kiosk.processing}
                  onclick={() => kiosk.updateAll()}
                >
                  <ArrowUpCircle class="size-4" />
                  {kiosk.updatable.length === 1
                    ? "1 Update installieren"
                    : `${kiosk.updatable.length} Updates installieren`}
                </Button>
              {/if}
            {/snippet}
          </PageHeader>

          {#if !isTauri()}
            <InfoBanner tone="info" class="mb-6">
              Browser-Vorschau — der opsi-Client-Dienst ist nur in der Desktop-App erreichbar.
            </InfoBanner>
          {:else if kiosk.status && !kiosk.status.connected && !kiosk.status.mock}
            <InfoBanner tone="warning" class="mb-6">
              Keine Verbindung zum opsi-Client-Dienst (opsiclientd). Läuft der Dienst auf diesem
              Computer?
            </InfoBanner>
          {/if}

          <div class="flex flex-wrap items-center gap-2 mb-6">
            <SearchInput bind:value={kiosk.search} placeholder="Apps suchen…" class="w-64" />
            <div class="flex items-center gap-1.5">
              {#each FILTERS as f (f.value)}
                <FilterChip
                  selected={kiosk.filter === f.value}
                  count={filterCount(f.value)}
                  onclick={() => (kiosk.filter = f.value)}
                >
                  {f.label}
                </FilterChip>
              {/each}
            </div>
          </div>

          {#if kiosk.loading}
            <EmptyState loading>Apps werden geladen…</EmptyState>
          {:else if kiosk.filtered.length === 0}
            <EmptyState icon={SearchX}>
              {kiosk.products.length === 0
                ? "Für diesen Computer ist keine Software-on-Demand-Gruppe freigegeben."
                : "Keine Apps für diesen Filter gefunden."}
            </EmptyState>
          {:else}
            <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-3 stagger">
              {#each kiosk.filtered as product (product.productId)}
                <ProductTile {product} onclick={() => (selectedId = product.productId)} />
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </main>

  <StatusFooter />

  <Toaster />
</div>
