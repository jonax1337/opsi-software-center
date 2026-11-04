<script lang="ts">
  import { Package } from "@lucide/svelte";
  import { kiosk } from "$lib/kiosk/store.svelte";
  import { cn } from "$lib/utils";

  type Size = "sm" | "md" | "lg";
  type Props = { productId: string; size?: Size };
  let { productId, size = "md" }: Props = $props();

  const SIZES: Record<Size, { box: string; image: string; fallback: string }> = {
    sm: { box: "size-10 rounded-md", image: "size-7", fallback: "size-5" },
    md: { box: "size-16 rounded-lg", image: "size-11", fallback: "size-8" },
    lg: { box: "size-24 rounded-xl", image: "size-16", fallback: "size-10" },
  };

  const icon = $derived(kiosk.icons[productId]);
</script>

<div
  class={cn(
    "grid place-items-center shrink-0 overflow-hidden border border-hairline bg-surface-3",
    SIZES[size].box,
  )}
>
  {#if icon}
    <img src={icon} alt="" class={cn("object-contain", SIZES[size].image)} />
  {:else}
    <Package class={cn("text-muted-foreground", SIZES[size].fallback)} />
  {/if}
</div>
