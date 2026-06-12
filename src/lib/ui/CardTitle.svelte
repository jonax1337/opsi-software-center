<script lang="ts">
  import type { Snippet } from "svelte";
  import { cn } from "$lib/utils";
  let { class: klass, children }: { class?: string; children?: Snippet } = $props();
</script>

<h3 class={cn("text-base font-semibold leading-none tracking-tight", klass)}>
  {@render children?.()}
</h3>
