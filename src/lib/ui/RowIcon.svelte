<script lang="ts">
  import type { Component, Snippet } from "svelte";
  import { cn } from "$lib/utils";

  type Props = {
    icon?: Component;
    tone?: "neutral" | "primary" | "muted" | "image";
    size?: "sm" | "md";
    iconClass?: string;
    class?: string;
    children?: Snippet;
  };

  let {
    icon: Icon,
    tone = "neutral",
    size = "md",
    iconClass,
    class: klass,
    children,
  }: Props = $props();

  const sizeCls = $derived(size === "sm" ? "size-7" : "size-9");
  const iconSize = $derived(size === "sm" ? "size-3.5" : "size-4");
  const container = $derived(
    {
      neutral: "bg-surface-3 border border-hairline text-muted-foreground",
      primary: "bg-primary/15 border border-primary/20 text-primary",
      muted: "bg-accent/60 border border-hairline text-muted-foreground",
      image: "bg-surface-3 border border-hairline overflow-hidden",
    }[tone],
  );
</script>

<div
  class={cn(
    "shrink-0 rounded-md flex items-center justify-center",
    sizeCls,
    container,
    klass,
  )}
>
  {#if Icon}
    <Icon class={cn(iconSize, iconClass)} />
  {/if}
  {@render children?.()}
</div>
