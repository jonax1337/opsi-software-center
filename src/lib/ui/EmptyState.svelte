<script lang="ts">
  import type { Component, Snippet } from "svelte";
  import { Loader2 } from "@lucide/svelte";
  import Card from "./Card.svelte";
  import { cn } from "$lib/utils";

  type Props = {
    loading?: boolean;
    icon?: Component;
    iconClass?: string;
    class?: string;
    children?: Snippet;
  };

  let { loading = false, icon: Icon, iconClass, class: klass, children }: Props = $props();
  const bare = $derived(!!loading || !!Icon);
</script>

{#if bare}
  <div class={cn("grid place-items-center py-24 text-sm text-muted-foreground", klass)}>
    {#if loading}
      <Loader2 class={cn("size-6 animate-spin mb-2", iconClass)} />
    {:else if Icon}
      <Icon class={cn("size-6 mb-2", iconClass)} />
    {/if}
    {@render children?.()}
  </div>
{:else}
  <Card class={cn("card-inset", klass)}>
    <div class="px-6 py-16 text-center text-sm text-muted-foreground">
      {@render children?.()}
    </div>
  </Card>
{/if}
