<script lang="ts">
  import { LayoutGrid, List, SearchX } from "@lucide/svelte";
  import { EmptyState, PageHeader, Select } from "$lib/ui";
  import ProductCollection, { type Layout } from "$lib/components/ProductCollection.svelte";
  import { kiosk } from "$lib/kiosk/store.svelte";
  import { selectProducts, type Filter } from "$lib/kiosk/product";
  import { cn } from "$lib/utils";

  let { onopen }: { onopen: (productId: string) => void } = $props();

  const LAYOUT_KEY = "apps-layout";
  const FILTERS: { value: Filter; label: string }[] = [
    { value: "all", label: "Alle Apps" },
    { value: "installed", label: "Installierte Apps" },
    { value: "available", label: "Nicht installierte Apps" },
    { value: "updates", label: "Updates verfügbar" },
  ];

  let layout = $state<Layout>(readLayout());

  /** Per-viewer convenience only — storage may be unavailable. */
  function readLayout(): Layout {
    try {
      return localStorage.getItem(LAYOUT_KEY) === "list" ? "list" : "tiles";
    } catch {
      return "tiles";
    }
  }

  function setLayout(next: Layout) {
    layout = next;
    try {
      localStorage.setItem(LAYOUT_KEY, next);
    } catch {
      // the choice just isn't remembered
    }
  }

  const query = $derived(kiosk.search.trim());
  const title = $derived(query ? `Ergebnisse für „${query}"` : "Apps");
  const filterOptions = $derived(
    FILTERS.map((f) => ({
      value: f.value,
      label: `${f.label} (${selectProducts(kiosk.products, f.value, kiosk.search).length})`,
    })),
  );

  function setFilter(value: string) {
    const match = FILTERS.find((f) => f.value === value);
    if (match) kiosk.filter = match.value;
  }
</script>

<PageHeader {title}>
  {kiosk.filtered.length === 1 ? "1 App" : `${kiosk.filtered.length} Apps`}
</PageHeader>

<div class="mb-5 flex flex-wrap items-center justify-between gap-3">
  <div class="flex items-center gap-2 text-sm">
    <span class="text-muted-foreground">Anzeigen:</span>
    <Select
      label="Apps filtern"
      value={kiosk.filter}
      options={filterOptions}
      onchange={setFilter}
    />
  </div>

  {#snippet layoutButton(value: Layout, label: string, Icon: typeof List)}
    <button
      type="button"
      aria-label={label}
      title={label}
      aria-pressed={layout === value}
      onclick={() => setLayout(value)}
      class={cn(
        "grid size-8 place-items-center rounded-sm transition-colors cursor-pointer outline-none focus-visible:ring-2 focus-visible:ring-ring",
        layout === value ? "bg-surface-4 text-foreground" : "text-muted-foreground hover:bg-surface-3",
      )}
    >
      <Icon class="size-4" />
    </button>
  {/snippet}
  <div class="flex items-center gap-1" role="group" aria-label="Ansicht">
    {@render layoutButton("tiles", "Kachelansicht", LayoutGrid)}
    {@render layoutButton("list", "Listenansicht", List)}
  </div>
</div>

{#if kiosk.filtered.length === 0}
  <EmptyState icon={SearchX}>
    {kiosk.products.length === 0
      ? "Für diesen Computer ist noch keine Software freigegeben."
      : "Keine Apps für diese Auswahl gefunden."}
  </EmptyState>
{:else}
  <ProductCollection products={kiosk.filtered} {layout} {onopen} />
{/if}
