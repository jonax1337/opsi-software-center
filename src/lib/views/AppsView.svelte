<script lang="ts">
  import { LayoutGrid, List, SearchX } from "@lucide/svelte";
  import { EmptyState, FilterChip, PageHeader } from "$lib/ui";
  import ProductCollection, { type Layout } from "$lib/components/ProductCollection.svelte";
  import { kiosk } from "$lib/kiosk/store.svelte";
  import { selectProducts, type Filter } from "$lib/kiosk/product";
  import { cn } from "$lib/utils";

  let { onopen }: { onopen: (productId: string) => void } = $props();

  const LAYOUT_KEY = "apps-layout";
  const FILTERS: { value: Filter; label: string }[] = [
    { value: "all", label: "Alle" },
    { value: "installed", label: "Installiert" },
    { value: "available", label: "Nicht installiert" },
    { value: "updates", label: "Updates" },
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
  const count = (filter: Filter) => selectProducts(kiosk.products, filter, kiosk.search).length;
</script>

<PageHeader {title}>
  {kiosk.filtered.length === 1 ? "1 App" : `${kiosk.filtered.length} Apps`}
</PageHeader>

<div class="mb-5 flex flex-wrap items-center justify-between gap-3">
  <div class="flex flex-wrap gap-2">
    {#each FILTERS as f (f.value)}
      <FilterChip
        selected={kiosk.filter === f.value}
        count={count(f.value)}
        onclick={() => (kiosk.filter = f.value)}
      >
        {f.label}
      </FilterChip>
    {/each}
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
