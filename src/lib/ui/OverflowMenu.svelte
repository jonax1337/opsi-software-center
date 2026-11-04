<script lang="ts" module>
  import type { Component } from "svelte";

  export type MenuItem = {
    label: string;
    icon: Component;
    onselect: () => void;
    disabled?: boolean;
    danger?: boolean;
  };
</script>

<script lang="ts">
  import { DropdownMenu } from "bits-ui";
  import { MoreHorizontal } from "$lib/ui/icons";

  /** Fluent "…" button with a MenuFlyout — the per-row menu of Company
   *  Portal's tables. */
  type Props = { label: string; items: MenuItem[] };
  let { label, items }: Props = $props();
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger
    aria-label={label}
    title={label}
    class="grid h-7 w-8 place-items-center rounded-sm border border-hairline-strong bg-card text-foreground transition-colors hover:bg-surface-3 outline-none focus-visible:ring-2 focus-visible:ring-ring cursor-pointer"
  >
    <MoreHorizontal class="size-4" />
  </DropdownMenu.Trigger>
  <DropdownMenu.Portal>
    <DropdownMenu.Content
      align="end"
      sideOffset={4}
      class="z-50 min-w-44 rounded-lg border border-hairline-strong bg-popover p-1 text-popover-foreground shadow-lg outline-none"
    >
      {#each items as item (item.label)}
        <DropdownMenu.Item
          disabled={item.disabled}
          onSelect={item.onselect}
          class="flex h-8 cursor-pointer items-center gap-2.5 rounded-sm px-3 text-sm outline-none select-none data-highlighted:bg-surface-3 data-disabled:pointer-events-none data-disabled:opacity-50 {item.danger
            ? 'text-destructive'
            : ''}"
        >
          <item.icon class="size-4 shrink-0" />
          {item.label}
        </DropdownMenu.Item>
      {/each}
    </DropdownMenu.Content>
  </DropdownMenu.Portal>
</DropdownMenu.Root>
