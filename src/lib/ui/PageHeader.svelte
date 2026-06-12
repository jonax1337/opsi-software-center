<script lang="ts">
  import type { Snippet } from "svelte";

  type Props = {
    title: string;
    description?: string;
    above?: Snippet;
    actions?: Snippet;
    children?: Snippet;
  };

  let { title, description, above, actions, children }: Props = $props();
</script>

<header class="mb-6 flex flex-wrap items-end justify-between gap-4">
  <div class="min-w-0">
    {#if above}{@render above()}{/if}
    <h1 class="text-3xl font-semibold tracking-tight">{title}</h1>
    {#if children}
      <p class="text-sm text-muted-foreground mt-1">{@render children()}</p>
    {:else if description}
      <p class="text-sm text-muted-foreground mt-1">{description}</p>
    {/if}
  </div>
  {#if actions}{@render actions()}{/if}
</header>
