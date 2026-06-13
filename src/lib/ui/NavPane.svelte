<script lang="ts" module>
  import type { Component } from "svelte";

  export type NavItem = {
    value: string;
    label: string;
    icon: Component;
    count?: number;
  };
</script>

<script lang="ts">
  import { Menu } from "@lucide/svelte";

  type Props = {
    items: NavItem[];
    footer?: NavItem[];
    value: string;
    onselect: (value: string) => void;
    open?: boolean;
  };

  let { items, footer = [], value, onselect, open = $bindable(true) }: Props = $props();

  // Refs to each nav button so the single selection indicator can travel to the
  // active one (WinUI: one pill that stretches in the direction of travel and
  // settles at the target — the "swoosh").
  const buttons: Record<string, HTMLButtonElement> = {};
  let indicator = $state<HTMLSpanElement>();
  let prev: { top: number; h: number } | null = null;

  $effect(() => {
    const el = buttons[value]; // tracked dep
    if (!el || !indicator) return;

    const to = { top: el.offsetTop + el.offsetHeight / 2 - 8, h: 16 };
    const reduce =
      typeof window !== "undefined" &&
      window.matchMedia("(prefers-reduced-motion: reduce)").matches;

    indicator.style.opacity = "1";
    if (prev && !reduce) {
      // Compact pill that glides, with only a subtle mid-travel elongation
      // (capped at 12px) — gliding, not a rubber band.
      const gap = Math.abs(to.top - prev.top);
      const stretch = Math.min(gap * 0.38, 16);
      const midTop = (prev.top + to.top) / 2 - stretch / 2;
      indicator.animate(
        [
          { top: `${prev.top}px`, height: `${prev.h}px` },
          { top: `${midTop}px`, height: `${16 + stretch}px`, offset: 0.5 },
          { top: `${to.top}px`, height: `${to.h}px` },
        ],
        { duration: 260, easing: "cubic-bezier(0.4, 0, 0.2, 1)" },
      );
    }
    // Resting state (= last keyframe), so there's no snap after the animation.
    indicator.style.top = `${to.top}px`;
    indicator.style.height = `${to.h}px`;
    prev = to;
  });
</script>

<nav
  class="relative shrink-0 flex flex-col gap-1 py-2 transition-[width] duration-[250ms] ease-[cubic-bezier(0.8,0,0.2,1)] select-none {open
    ? 'w-64'
    : 'w-[52px]'}"
  aria-label="Hauptnavigation"
>
  <!-- Single moving selection indicator: one 3px accent pill, flush against the
       selected item's grey fill (left-2 = the item's mx-2 edge). Position +
       stretch are driven imperatively via the Web Animations API. -->
  <span
    bind:this={indicator}
    class="pointer-events-none absolute left-2 w-[3px] rounded-full bg-primary opacity-0"
    style="height: 16px;"
  ></span>

  <!-- Hamburger toggle — WinUI NavigationView pane button -->
  <button
    type="button"
    class="mx-2 mb-1 grid h-10 w-9 place-items-center rounded-sm text-foreground hover:bg-surface-3 active:bg-surface-1 transition-colors outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background"
    aria-label={open ? "Navigation einklappen" : "Navigation ausklappen"}
    aria-expanded={open}
    onclick={() => (open = !open)}
  >
    <Menu class="size-5" />
  </button>

  {#snippet navButton(item: NavItem)}
    {@const selected = item.value === value}
    <button
      type="button"
      bind:this={buttons[item.value]}
      title={!open ? item.label : undefined}
      aria-current={selected ? "page" : undefined}
      onclick={() => onselect(item.value)}
      class="relative mx-2 flex h-10 items-center rounded-sm text-sm transition-colors outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background {open
        ? 'gap-3 px-3'
        : 'justify-center px-0'} {selected
        ? 'bg-surface-3 font-semibold text-foreground'
        : 'font-normal text-foreground/90 hover:bg-surface-2'}"
    >
      <item.icon class="size-5 shrink-0 {selected ? 'text-primary' : 'text-muted-foreground'}" />
      {#if open}
        <span class="truncate">{item.label}</span>
        {#if item.count !== undefined}
          <span class="ml-auto text-xs tabular-nums text-muted-foreground">{item.count}</span>
        {/if}
      {/if}
    </button>
  {/snippet}

  <div class="flex flex-col gap-1">
    {#each items as item (item.value)}
      {@render navButton(item)}
    {/each}
  </div>

  {#if footer.length}
    <div class="mt-auto flex flex-col gap-1">
      {#each footer as item (item.value)}
        {@render navButton(item)}
      {/each}
    </div>
  {/if}
</nav>
