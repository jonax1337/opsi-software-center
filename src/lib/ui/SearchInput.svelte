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
  <Search class="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" />
  <input
    type="text"
    bind:value
    {placeholder}
    class="w-full h-9 pl-9 pr-3 rounded-md border border-input bg-card text-sm outline-none focus-visible:ring-ring/50 focus-visible:ring-[3px] focus-visible:border-ring"
  />
</div>
