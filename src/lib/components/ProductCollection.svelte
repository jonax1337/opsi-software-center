<script lang="ts" module>
  /** `row`: a single row of cards, as many as fit (Company Portal home); the
   *  rest is reached via "Alle anzeigen". */
  export type Layout = "tiles" | "list" | "row";
</script>

<script lang="ts">
  import ProductTable from "./ProductTable.svelte";
  import ProductTile from "./ProductTile.svelte";
  import type { KioskProduct } from "$lib/kiosk/types";

  type Props = {
    products: KioskProduct[];
    layout?: Layout;
    onopen: (productId: string) => void;
  };
  let { products, layout = "tiles", onopen }: Props = $props();
</script>

{#if layout === "list"}
  <ProductTable {products} {onopen} />
{:else}
  <!-- Fixed-width cards like Company Portal. For `row`, wrapped rows collapse
       to zero height and are clipped; -m-1/p-1 keeps the card shadows. -->
  <div
    class="grid grid-cols-[repeat(auto-fill,9.5rem)] gap-3 stagger {layout === 'row'
      ? '-m-1 auto-rows-[0] grid-rows-1 gap-y-0 overflow-hidden p-1'
      : ''}"
  >
    {#each products as product (product.productId)}
      <ProductTile {product} onclick={() => onopen(product.productId)} />
    {/each}
  </div>
{/if}
