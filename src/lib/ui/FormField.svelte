<script lang="ts">
  import type { Snippet } from "svelte";
  import { cn } from "$lib/utils";

  type Props = {
    label: string;
    hint?: string;
    class?: string;
    children?: Snippet;
  };

  let { label, hint, class: klass, children }: Props = $props();
</script>

<label class={cn("flex flex-col gap-1.5", klass)}>
  <span class="text-xs font-medium text-muted-foreground">{label}</span>
  {@render children?.()}
  {#if hint}
    <span class="text-[11px] text-muted-foreground/70 leading-relaxed">{hint}</span>
  {/if}
</label>
