<script lang="ts">
  import AppIcon from "./AppIcon.svelte";
  import ProductStatus from "./ProductStatus.svelte";
  import { displayName, displayVersion } from "$lib/kiosk/product";
  import type { KioskProduct } from "$lib/kiosk/types";

  type Props = { product: KioskProduct; onclick: () => void };
  let { product, onclick }: Props = $props();
</script>

<!-- Company Portal app card: raised card, icon top-left, bold name, grey
     version line (where CP shows the publisher — opsi has none) and the status. -->
<button
  type="button"
  {onclick}
  class="card-inset flex h-48 w-full flex-col items-start gap-3 rounded-lg bg-card p-4 text-left transition-[background-color,box-shadow] duration-150 hover:bg-surface-3 active:bg-surface-2 cursor-pointer outline-none focus-visible:ring-2 focus-visible:ring-ring"
>
  <AppIcon productId={product.productId} />
  <div class="w-full min-w-0">
    <div class="line-clamp-2 text-sm font-semibold leading-snug">{displayName(product)}</div>
    <div class="truncate text-xs text-muted-foreground">{displayVersion(product)}</div>
    <div class="mt-1 empty:hidden"><ProductStatus {product} /></div>
  </div>
</button>
