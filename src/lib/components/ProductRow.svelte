<script lang="ts">
  import { ChevronRight } from "@lucide/svelte";
  import AppIcon from "./AppIcon.svelte";
  import ProductStatus from "./ProductStatus.svelte";
  import { displayName, isInstalled } from "$lib/kiosk/product";
  import type { KioskProduct } from "$lib/kiosk/types";

  type Props = { product: KioskProduct; onclick: () => void };
  let { product, onclick }: Props = $props();

  const version = $derived(isInstalled(product) ? product.installedVerStr : product.versionStr);
</script>

<button
  type="button"
  {onclick}
  class="flex w-full items-center gap-4 border-b border-hairline px-4 py-3 text-left last:border-b-0 transition-colors hover:bg-surface-3 cursor-pointer outline-none focus-visible:bg-surface-3"
>
  <AppIcon productId={product.productId} size="sm" />
  <div class="min-w-0 flex-1">
    <div class="truncate text-sm font-medium">{displayName(product)}</div>
    <div class="truncate font-mono text-xs tabular-nums text-muted-foreground">{version}</div>
  </div>
  <div class="hidden w-40 shrink-0 sm:block">
    <ProductStatus {product} />
  </div>
  <ChevronRight class="size-4 shrink-0 text-muted-foreground" />
</button>
