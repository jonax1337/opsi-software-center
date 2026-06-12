<script lang="ts">
  import type { Component, Snippet } from "svelte";
  import { Loader2 } from "@lucide/svelte";
  import Card from "./Card.svelte";
  import CardContent from "./CardContent.svelte";
  import { cn } from "$lib/utils";

  type Props = {
    label: string;
    icon: Component;
    value?: number | string;
    total?: number | string;
    loading?: boolean;
    hint?: string;
    class?: string;
    footer?: Snippet;
  };

  let {
    label,
    icon: Icon,
    value,
    total,
    loading = false,
    hint,
    class: klass,
    footer,
  }: Props = $props();
</script>

<Card class={cn("card-inset", klass)}>
  <CardContent>
    <div class="flex items-center justify-between">
      <span class="text-xs text-muted-foreground uppercase tracking-wider">{label}</span>
      <Icon class="size-4 text-muted-foreground" />
    </div>
    <div class="flex items-baseline gap-2 mt-2">
      <span class="text-3xl font-semibold tabular-nums">
        {#if loading}
          <Loader2 class="size-6 animate-spin text-muted-foreground" />
        {:else}
          {value}
        {/if}
      </span>
      {#if !loading && total !== undefined}
        <span class="text-sm text-muted-foreground">/ {total}</span>
      {/if}
    </div>
    {#if hint}
      <div class="text-xs text-muted-foreground mt-1">{hint}</div>
    {/if}
    {#if footer}
      {@render footer()}
    {/if}
  </CardContent>
</Card>
