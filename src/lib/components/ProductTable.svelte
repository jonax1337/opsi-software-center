<script lang="ts">
  import type { Snippet } from "svelte";
  import AppIcon from "./AppIcon.svelte";
  import ProductStatus from "./ProductStatus.svelte";
  import { kiosk } from "$lib/kiosk/store.svelte";
  import { displayName, displayVersion, productState, publisherOf } from "$lib/kiosk/product";
  import type { KioskProduct } from "$lib/kiosk/types";

  /** Company Portal table (Apps list view, Downloads und Updates): Name,
   *  Version, Herausgeber, Status, optional per-row actions. The whole row
   *  opens the detail page; the actions cell sits above that click target. */
  type Props = {
    products: KioskProduct[];
    onopen: (productId: string) => void;
    actions?: Snippet<[KioskProduct]>;
  };
  let { products, onopen, actions }: Props = $props();

  const COLUMNS = "grid-cols-[minmax(0,2.2fr)_minmax(0,1fr)_minmax(0,1.4fr)_minmax(0,1.3fr)_3rem]";
</script>

<div class="card-inset overflow-hidden rounded-lg bg-card text-sm" role="table">
  <div
    role="row"
    class="grid {COLUMNS} items-center gap-4 border-b border-hairline px-4 py-2.5 text-xs text-muted-foreground"
  >
    <span role="columnheader" class="pl-11">Name</span>
    <span role="columnheader">Version</span>
    <span role="columnheader">Herausgeber</span>
    <span role="columnheader">Status</span>
    <span role="columnheader" class="sr-only">Aktionen</span>
  </div>
  {#each products as product (product.productId)}
    {@const state = productState(product, kiosk.processing)}
    <div
      role="row"
      class="relative grid {COLUMNS} items-center gap-4 border-b border-hairline px-4 py-2 last:border-b-0 transition-colors hover:bg-surface-3"
    >
      <span role="cell" class="flex min-w-0 items-center gap-3">
        <AppIcon productId={product.productId} size="xs" />
        <button
          type="button"
          onclick={() => onopen(product.productId)}
          class="truncate text-left outline-none after:absolute after:inset-0 focus-visible:after:ring-2 focus-visible:after:ring-inset focus-visible:after:ring-ring cursor-pointer"
        >
          {displayName(product)}
        </button>
      </span>
      <span role="cell" class="truncate text-muted-foreground">{displayVersion(product)}</span>
      <span role="cell" class="truncate text-muted-foreground">
        {publisherOf(product, kiosk.catalog) || "—"}
      </span>
      <span role="cell" class="min-w-0">
        {#if state === "available"}
          <span class="text-xs text-muted-foreground">Verfügbar</span>
        {:else}
          <ProductStatus {product} />
        {/if}
        {#if state === "running"}
          <span class="progress-indeterminate mt-1.5 block h-0.5 overflow-hidden rounded-full bg-hairline"></span>
        {/if}
      </span>
      <span role="cell" class="relative z-10 flex justify-end">
        {#if actions}{@render actions(product)}{/if}
      </span>
    </div>
  {/each}
</div>
