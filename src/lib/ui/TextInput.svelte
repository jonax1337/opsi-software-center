<script lang="ts">
  import { cn } from "$lib/utils";

  type Props = {
    value?: string;
    placeholder?: string;
    type?: "text" | "email" | "password" | "url";
    mono?: boolean;
    readonly?: boolean;
    disabled?: boolean;
    class?: string;
    oninput?: (e: Event) => void;
    onchange?: (e: Event) => void;
  };

  let {
    value = $bindable(""),
    placeholder,
    type = "text",
    mono = false,
    readonly = false,
    disabled = false,
    class: klass,
    oninput,
    onchange,
  }: Props = $props();
</script>

<input
  {type}
  bind:value
  {placeholder}
  {readonly}
  {disabled}
  {oninput}
  {onchange}
  class={cn(
    "w-full h-9 rounded-md border border-input bg-card px-3 text-sm outline-none focus-visible:ring-ring/50 focus-visible:ring-[3px] focus-visible:border-ring disabled:opacity-60",
    mono && "font-mono",
    klass,
  )}
/>
