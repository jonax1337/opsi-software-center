<script lang="ts">
  import { Search } from "@lucide/svelte";
  import { cn } from "$lib/utils";

  type Props = {
    value?: string;
    placeholder?: string;
    class?: string;
  };

  let {
    value = $bindable(""),
    placeholder = "Filter…",
    class: klass,
  }: Props = $props();
</script>

<div class={cn("relative", klass)}>
  <Search class="absolute left-3.5 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" />
  <input
    type="text"
    bind:value
    {placeholder}
    class="w-full h-10 pl-10 pr-3.5 rounded-lg border border-hairline-strong bg-surface-2 text-sm outline-none transition-[box-shadow,border-color,background-color] hover:bg-surface-3 focus-visible:border-primary/40 focus-visible:[box-shadow:inset_0_-2px_0_0_var(--primary)]"
  />
</div>
