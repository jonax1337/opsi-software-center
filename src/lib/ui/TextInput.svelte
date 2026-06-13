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
    "w-full h-8 rounded-sm border border-hairline-strong bg-surface-2 px-3 text-sm outline-none transition-[box-shadow,border-color,background-color] hover:bg-surface-3 focus-visible:border-primary/40 focus-visible:[box-shadow:inset_0_-2px_0_0_var(--primary)] disabled:opacity-60",
    mono && "font-mono",
    klass,
  )}
/>
