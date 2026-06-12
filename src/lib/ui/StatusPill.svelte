<script lang="ts">
  import type { Component, Snippet } from "svelte";
  import { cn } from "$lib/utils";

  type Props = {
    tone?: "neutral" | "muted" | "success" | "warning";
    icon?: Component;
    onclick?: () => void;
    disabled?: boolean;
    title?: string;
    class?: string;
    style?: string;
    children?: Snippet;
  };

  let {
    tone = "neutral",
    icon: Icon,
    onclick,
    disabled = false,
    title,
    class: klass,
    style,
    children,
  }: Props = $props();

  const base =
    "inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold uppercase tracking-wider border";
  const tones = {
    neutral: "bg-foreground/10 text-foreground border-foreground/15",
    muted: "bg-muted/50 text-muted-foreground border-muted/60",
    success: "bg-success/15 text-success border-success/30",
    warning: "bg-amber-500/15 text-amber-700 dark:text-amber-300 border-amber-500/30",
  };
  const hovers = {
    neutral: "hover:bg-foreground/15",
    muted: "hover:bg-muted/70",
    success: "hover:bg-success/25",
    warning: "hover:bg-amber-500/25",
  };
  const toneCls = $derived(tones[tone]);
  const hoverCls = $derived(onclick ? hovers[tone] : "");
</script>

{#if onclick}
  <button
    type="button"
    {onclick}
    {disabled}
    {title}
    {style}
    class={cn(base, toneCls, hoverCls, "transition-colors disabled:opacity-50", klass)}
  >
    {#if Icon}<Icon class="size-3" />{/if}
    {@render children?.()}
  </button>
{:else}
  <span class={cn(base, toneCls, klass)} {title} {style}>
    {#if Icon}<Icon class="size-3" />{/if}
    {@render children?.()}
  </span>
{/if}
