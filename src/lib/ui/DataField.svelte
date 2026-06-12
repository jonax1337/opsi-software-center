<script lang="ts">
  import type { Snippet } from "svelte";
  import { cn } from "$lib/utils";

  type Props = {
    label: string;
    value?: string | number;
    mono?: boolean;
    class?: string;
    children?: Snippet;
  };

  let { label, value, mono = false, class: klass, children }: Props = $props();
</script>

<div class={klass}>
  <div class="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/70">
    {label}
  </div>
  <div class={cn("mt-1", mono ? "font-mono text-sm tabular-nums break-all" : "text-sm")}>
    {#if children}{@render children()}{:else}{value ?? "—"}{/if}
  </div>
</div>
