<script lang="ts">
  import type { Snippet } from "svelte";
  import { cn } from "$lib/utils";
  let { class: klass, children }: { class?: string; children?: Snippet } = $props();
</script>

<div class={cn("flex flex-col gap-1.5 px-6", klass)}>
  {@render children?.()}
</div>
