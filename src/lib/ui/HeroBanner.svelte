<script lang="ts">
  import type { Snippet } from "svelte";
  import { cn } from "$lib/utils";

  type Props = {
    tone?: "default" | "success" | "warning" | "none";
    withDots?: boolean;
    class?: string;
    children?: Snippet;
  };

  let { tone = "default", withDots = false, class: klass, children }: Props = $props();
  const toneCls = $derived(
    {
      default: "hero-glow",
      success: "hero-glow-success",
      warning: "hero-glow-warning",
      none: "",
    }[tone],
  );
</script>

<section
  class={cn(
    "relative overflow-hidden rounded-2xl border border-hairline-strong bg-card/70 backdrop-blur-xl shadow-sm mb-6",
    toneCls,
    klass,
  )}
>
  {#if withDots}
    <div
      class="absolute inset-0 -z-10 opacity-[0.04] [background-image:radial-gradient(circle_at_1px_1px,_currentColor_1px,_transparent_0)] [background-size:16px_16px]"
    ></div>
  {/if}
  {@render children?.()}
</section>
