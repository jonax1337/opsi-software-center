<script lang="ts">
  import AppIcon from "./AppIcon.svelte";
  import ProductStatus from "./ProductStatus.svelte";
  import { displayName, displayVersion } from "$lib/kiosk/product";
  import type { KioskProduct } from "$lib/kiosk/types";

  type Props = { product: KioskProduct; onclick: () => void };
  let { product, onclick }: Props = $props();
</script>

<!-- Company Portal tile: borderless, icon on top, name and a grey secondary
     line below (the version, where CP shows the publisher); the surface only
     appears on hover/focus. -->
<button
  type="button"
  {onclick}
  class="flex flex-col items-start gap-2.5 rounded-md p-3 text-left transition-colors duration-150 hover:bg-surface-3 active:bg-surface-2 cursor-pointer outline-none focus-visible:ring-2 focus-visible:ring-ring"
>
  <AppIcon productId={product.productId} />
  <div class="w-full min-w-0">
    <div class="line-clamp-2 text-sm leading-snug">{displayName(product)}</div>
    <div class="truncate text-xs text-muted-foreground">{displayVersion(product)}</div>
    <div class="mt-1 empty:hidden"><ProductStatus {product} /></div>
  </div>
</button>
