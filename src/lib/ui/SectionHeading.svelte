<script lang="ts">
  import type { Snippet } from "svelte";
  import { cn } from "$lib/utils";

  type Props = {
    title: string;
    description?: string;
    hint?: string;
    actions?: Snippet;
    inline?: Snippet;
    level?: "h2" | "h3" | "h4";
    class?: string;
    children?: Snippet;
  };

  let {
    title,
    description,
    hint,
    actions,
    inline,
    level = "h2",
    class: klass,
    children,
  }: Props = $props();

  /** Fluent "Subtitle" ramp (20px semibold, sentence case), as in Company Portal. */
  const heading = "text-xl font-semibold leading-7 text-foreground";
  const hasRight = $derived(!!actions || !!hint);
  const hasDescription = $derived(!!description || !!children);
</script>

{#if hasDescription}
  <div class={cn("mb-3", klass)}>
    <div class="flex items-center justify-between">
      <svelte:element this={level} class={heading}>
        {title}{#if inline}{@render inline()}{/if}
      </svelte:element>
      {#if actions}
        {@render actions()}
      {:else if hint}
        <span class="text-[11px] text-muted-foreground">{hint}</span>
      {/if}
    </div>
    <p class="text-sm text-muted-foreground mt-1 leading-relaxed">
      {#if children}{@render children()}{:else}{description}{/if}
    </p>
  </div>
{:else if hasRight}
  <div class={cn("flex items-center justify-between mb-3", klass)}>
    <svelte:element this={level} class={heading}>
      {title}{#if inline}{@render inline()}{/if}
    </svelte:element>
    {#if actions}
      {@render actions()}
    {:else if hint}
      <span class="text-[11px] text-muted-foreground">{hint}</span>
    {/if}
  </div>
{:else}
  <svelte:element this={level} class={cn(heading, "mb-3", klass)}>
    {title}{#if inline}{@render inline()}{/if}
  </svelte:element>
{/if}
