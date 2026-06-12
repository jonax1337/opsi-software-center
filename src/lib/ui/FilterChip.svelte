<script lang="ts">
  import type { Snippet } from "svelte";
  import { cn } from "$lib/utils";

  type Props = {
    selected?: boolean;
    count?: number;
    onclick?: () => void;
    disabled?: boolean;
    class?: string;
    children?: Snippet;
  };

  let {
    selected = false,
    count,
    onclick,
    disabled = false,
    class: klass,
    children,
  }: Props = $props();
</script>

<button
  type="button"
  {onclick}
  {disabled}
  class={cn(
    "inline-flex items-center gap-1.5 px-3 h-8 rounded-md text-xs font-medium transition-colors border disabled:opacity-50",
    selected
      ? "border-primary bg-primary/10 text-primary"
      : "border-input hover:bg-accent/40 text-muted-foreground",
    klass,
  )}
>
  {@render children?.()}
  {#if count !== undefined}
    <span class="tabular-nums text-[10px] opacity-70">({count})</span>
  {/if}
</button>
