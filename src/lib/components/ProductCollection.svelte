<script lang="ts" module>
  export type Layout = "tiles" | "list";
</script>

<script lang="ts">
  import { ListCard } from "$lib/ui";
  import ProductRow from "./ProductRow.svelte";
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
  <ListCard>
    {#each products as product (product.productId)}
      <ProductRow {product} onclick={() => onopen(product.productId)} />
    {/each}
  </ListCard>
{:else}
  <div class="grid grid-cols-[repeat(auto-fill,minmax(9rem,1fr))] gap-1 stagger">
    {#each products as product (product.productId)}
      <ProductTile {product} onclick={() => onopen(product.productId)} />
    {/each}
  </div>
{/if}
