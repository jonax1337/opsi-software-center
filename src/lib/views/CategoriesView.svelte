<script lang="ts">
  import { EmptyState, PageHeader } from "$lib/ui";
  import AppIcon from "$lib/components/AppIcon.svelte";
  import { kiosk } from "$lib/kiosk/store.svelte";
  import { categoriesOf } from "$lib/kiosk/product";

  /** Company Portal "App-Kategorien": one card per category (from catalog.json);
   *  a click shows the category on the Apps page. */
  let { onselect }: { onselect: (category: string) => void } = $props();

  const ICON_PREVIEW = 4;
  const categories = $derived(categoriesOf(kiosk.products, kiosk.catalog));
</script>

<PageHeader title="App-Kategorien">
  Von Ihrer IT nach Einsatzbereich sortiert.
</PageHeader>

{#if categories.length}
  <div class="grid grid-cols-[repeat(auto-fill,minmax(14rem,1fr))] gap-3 stagger">
    {#each categories as category (category.name)}
      <button
        type="button"
        onclick={() => onselect(category.name)}
        class="card-inset flex flex-col gap-4 rounded-lg bg-card p-5 text-left transition-colors hover:bg-surface-3 cursor-pointer outline-none focus-visible:ring-2 focus-visible:ring-ring"
      >
        <span class="flex gap-2">
          {#each category.products.slice(0, ICON_PREVIEW) as p (p.productId)}
            <AppIcon productId={p.productId} size="sm" />
          {/each}
        </span>
        <span>
          <span class="block text-base font-semibold">{category.name}</span>
          <span class="block text-xs text-muted-foreground">
            {category.products.length === 1 ? "1 App" : `${category.products.length} Apps`}
          </span>
        </span>
      </button>
    {/each}
  </div>
{:else}
  <EmptyState>Ihre IT hat noch keine App-Kategorien festgelegt.</EmptyState>
{/if}
