<script lang="ts">
  import { Box } from "$lib/ui/icons";
  import { kiosk } from "$lib/kiosk/store.svelte";
  import { cn } from "$lib/utils";

  type Size = "xs" | "sm" | "md" | "lg";
  type Props = { productId: string; size?: Size };
  let { productId, size = "md" }: Props = $props();

  const SIZES: Record<Size, { box: string; fallback: string }> = {
    xs: { box: "size-6 rounded-sm", fallback: "size-3.5" },
    sm: { box: "size-10 rounded-md", fallback: "size-5" },
    md: { box: "size-16 rounded-lg", fallback: "size-8" },
    lg: { box: "size-24 rounded-xl", fallback: "size-10" },
  };

  const icon = $derived(kiosk.icons[productId]);
</script>

<!-- Company Portal shows the app's own icon full-size; only the fallback
     glyph sits on a neutral plate. -->
{#if icon}
  <img src={icon} alt="" class={cn("shrink-0 object-contain", SIZES[size].box)} />
{:else}
  <div
    class={cn(
      "grid shrink-0 place-items-center border border-hairline bg-surface-3",
      SIZES[size].box,
    )}
  >
    <Box class={cn("text-muted-foreground", SIZES[size].fallback)} />
  </div>
{/if}
