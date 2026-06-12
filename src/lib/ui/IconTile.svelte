<script lang="ts">
  import type { Snippet } from "svelte";
  import { cn } from "$lib/utils";

  type Props = {
    size?: "sm" | "md" | "lg";
    radius?: "md" | "lg";
    interactive?: boolean;
    class?: string;
    children?: Snippet;
  };

  let {
    size = "md",
    radius = "lg",
    interactive = false,
    class: klass,
    children,
  }: Props = $props();

  const sizeCls = $derived({ sm: "size-8", md: "size-9", lg: "size-10" }[size]);
  const radiusCls = $derived({ md: "rounded-md", lg: "rounded-lg" }[radius]);
</script>

<div
  class={cn(
    "grid place-items-center shrink-0 bg-surface-4 text-foreground/80 ring-1 ring-inset ring-foreground/5",
    sizeCls,
    radiusCls,
    interactive && "group-hover:text-primary group-hover:bg-primary/10 transition-colors",
    klass,
  )}
>
  {@render children?.()}
</div>
