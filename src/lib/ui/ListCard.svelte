<script lang="ts">
  import type { Snippet } from "svelte";
  import Card from "./Card.svelte";
  import { cn } from "$lib/utils";

  type Props = {
    class?: string;
    children?: Snippet;
  };

  let { class: klass, children }: Props = $props();
</script>

<Card class={cn("overflow-hidden gap-0 py-0 card-inset", klass)}>
  {@render children?.()}
</Card>
