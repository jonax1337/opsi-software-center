<script lang="ts" module>
  export type SelectOption = { value: string; label: string };
</script>

<script lang="ts">
  import { Select } from "bits-ui";
  import { Checkmark, ChevronDown } from "$lib/ui/icons";
  import { cn } from "$lib/utils";

  /** Fluent ComboBox: 32px trigger with chevron, popup list with an accent
   *  pill on the selected item (WinUI ComboBox / Company Portal pickers). */
  type Props = {
    value: string;
    options: SelectOption[];
    onchange: (value: string) => void;
    label: string;
    class?: string;
  };
  let { value, options, onchange, label, class: klass }: Props = $props();

  const selectedLabel = $derived(options.find((o) => o.value === value)?.label ?? "");
</script>

<Select.Root
  type="single"
  {value}
  items={options}
  onValueChange={(v) => v && onchange(v)}
>
  <Select.Trigger
    aria-label={label}
    class={cn(
      "inline-flex h-8 min-w-40 items-center justify-between gap-3 rounded-sm border border-hairline-strong bg-surface-2 px-3 text-sm transition-colors hover:bg-surface-3 outline-none focus-visible:ring-2 focus-visible:ring-ring cursor-pointer",
      klass,
    )}
  >
    <span class="truncate">{selectedLabel}</span>
    <ChevronDown class="size-4 shrink-0 text-muted-foreground" />
  </Select.Trigger>
  <Select.Portal>
    <Select.Content
      sideOffset={4}
      class="z-50 min-w-(--bits-select-anchor-width) rounded-lg border border-hairline-strong bg-popover p-1 text-popover-foreground shadow-lg outline-none"
    >
      <Select.Viewport>
        {#each options as option (option.value)}
          <Select.Item
            value={option.value}
            label={option.label}
            class="relative flex h-8 cursor-pointer items-center gap-2 rounded-sm pl-3 pr-8 text-sm outline-none select-none data-highlighted:bg-surface-3"
          >
            {#snippet children({ selected })}
              {#if selected}
                <span class="absolute left-0 h-4 w-[3px] rounded-full bg-primary"></span>
              {/if}
              <span class="truncate">{option.label}</span>
              {#if selected}
                <Checkmark class="absolute right-2 size-4 text-muted-foreground" />
              {/if}
            {/snippet}
          </Select.Item>
        {/each}
      </Select.Viewport>
    </Select.Content>
  </Select.Portal>
</Select.Root>
