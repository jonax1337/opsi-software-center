<script lang="ts" module>
  export type Layout = "tiles" | "list";
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
  <div class="grid grid-cols-[repeat(auto-fill,minmax(9.5rem,1fr))] gap-3 stagger">
    {#each products as product (product.productId)}
      <ProductTile {product} onclick={() => onopen(product.productId)} />
    {/each}
  </div>
{/if}
