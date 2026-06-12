<script lang="ts">
  import type { Snippet } from "svelte";
  import { cn } from "$lib/utils";

  type Props = {
    align?: "start" | "center";
    density?: "sm" | "md";
    interactive?: boolean;
    class?: string;
    children?: Snippet;
  };

  let {
    align = "start",
    density = "sm",
    interactive = false,
    class: klass,
    children,
  }: Props = $props();

  const alignCls = $derived(align === "center" ? "items-center" : "items-start");
  const pyCls = $derived(density === "md" ? "py-4" : "py-3");
</script>

<div
  class={cn(
    "flex gap-3 px-5 border-b last:border-b-0 hover:bg-accent/30 transition-colors",
    alignCls,
    pyCls,
    interactive && "cursor-pointer",
    klass,
  )}
>
  {@render children?.()}
</div>
