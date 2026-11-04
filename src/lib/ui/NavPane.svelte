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
  import { Navigation } from "$lib/ui/icons";

  /** WinUI NavigationView in LeftCompact mode: a 52px icon rail that keeps its
   *  place in the layout; the hamburger opens the full pane as an overlay above
   *  the content (light-dismiss), so the page never reflows. Icons stay put
   *  while the pane widens; only the labels fade in. */
  type Props = {
    items: NavItem[];
    footer?: NavItem[];
    value: string;
    onselect: (value: string) => void;
    open?: boolean;
  };

  let { items, footer = [], value, onselect, open = $bindable(false) }: Props = $props();

  const INDICATOR_HEIGHT = 16;
  /** Beyond ~2 rows the pill no longer glides along the rail (that reads as
   *  "sliding all the way down") — it shrinks away and grows at the target. */
  const MAX_GLIDE_DISTANCE = 96;
  const GLIDE = { duration: 260, easing: "cubic-bezier(0.4, 0, 0.2, 1)" };
  const SHRINK_GROW = { duration: 300, easing: "cubic-bezier(0.8, 0, 0.2, 1)" };

  const buttons: Record<string, HTMLButtonElement> = {};
  let indicator = $state<HTMLSpanElement>();
  let prev: number | null = null;

  function select(next: string) {
    open = false;
    onselect(next);
  }

  $effect(() => {
    const el = buttons[value]; // tracked dep
    if (!el || !indicator) return;

    const to = el.offsetTop + el.offsetHeight / 2 - INDICATOR_HEIGHT / 2;
    const reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

    indicator.style.opacity = "1";
    if (prev !== null && prev !== to && !reduce) animateIndicator(indicator, prev, to);
    // Resting state (= last keyframe), so there's no snap after the animation.
    indicator.style.top = `${to}px`;
    indicator.style.height = `${INDICATOR_HEIGHT}px`;
    prev = to;
  });

  function animateIndicator(pill: HTMLSpanElement, from: number, to: number) {
    const h = `${INDICATOR_HEIGHT}px`;
    const distance = Math.abs(to - from);
    if (distance <= MAX_GLIDE_DISTANCE) {
      // Glide with a subtle mid-travel elongation — gliding, not a rubber band.
      const stretch = Math.min(distance * 0.38, INDICATOR_HEIGHT);
      const mid = (from + to) / 2 - stretch / 2;
      pill.animate(
        [
          { top: `${from}px`, height: h },
          { top: `${mid}px`, height: `${INDICATOR_HEIGHT + stretch}px`, offset: 0.5 },
          { top: `${to}px`, height: h },
        ],
        GLIDE,
      );
      return;
    }
    // Shrink to nothing at the old item, jump while invisible, grow at the new one.
    const center = INDICATOR_HEIGHT / 2;
    pill.animate(
      [
        { top: `${from}px`, height: h },
        { top: `${from + center}px`, height: "0px", offset: 0.45 },
        { top: `${to + center}px`, height: "0px", offset: 0.55 },
        { top: `${to}px`, height: h },
      ],
      SHRINK_GROW,
    );
  }
</script>

<svelte:window onkeydown={(e) => open && e.key === "Escape" && (open = false)} />

{#if open}
  <!-- Light dismiss: a click anywhere outside the open pane closes it. -->
  <button
    type="button"
    tabindex="-1"
    aria-label="Navigation schließen"
    class="fixed inset-0 z-20 cursor-default"
    onclick={() => (open = false)}
  ></button>
{/if}

<!-- The wrapper reserves the rail's width; the nav itself may grow over the page. -->
<div class="relative w-[52px] shrink-0">
  <nav
    class="absolute inset-y-0 left-0 z-30 flex flex-col gap-1 overflow-hidden py-2 select-none transition-[width,background-color,box-shadow] duration-200 ease-[cubic-bezier(0.8,0,0.2,1)] {open
      ? 'w-64 border-r border-hairline bg-popover/95 shadow-[0_8px_32px_rgb(0_0_0/0.28)] backdrop-blur-xl'
      : 'w-[52px] bg-transparent'}"
    aria-label="Hauptnavigation"
  >
    <!-- Single selection indicator: a 3px accent pill at the item's left edge,
         moved imperatively via the Web Animations API. -->
    <span
      bind:this={indicator}
      class="pointer-events-none absolute left-1.5 z-10 w-[3px] rounded-full bg-primary opacity-0"
      style="height: {INDICATOR_HEIGHT}px;"
    ></span>

    <!-- Hamburger: same 40px box and left offset as the items, so nothing shifts. -->
    <button
      type="button"
      class="mx-1.5 mb-1 grid h-9 w-10 shrink-0 place-items-center rounded-sm text-foreground hover:bg-surface-3 active:bg-surface-1 transition-colors outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring"
      aria-label={open ? "Navigation einklappen" : "Navigation ausklappen"}
      aria-expanded={open}
      onclick={() => (open = !open)}
    >
      <Navigation class="size-5" />
    </button>

    {#snippet navButton(item: NavItem)}
      {@const selected = item.value === value}
      <button
        type="button"
        bind:this={buttons[item.value]}
        title={!open ? item.label : undefined}
        aria-current={selected ? "page" : undefined}
        onclick={() => select(item.value)}
        class="relative mx-1.5 flex h-9 shrink-0 items-center gap-3 rounded-sm px-2.5 text-sm transition-colors outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring {selected
          ? 'bg-surface-3 font-semibold text-foreground'
          : 'font-normal text-foreground/90 hover:bg-surface-2'}"
      >
        <item.icon class="size-5 shrink-0 {selected ? 'text-primary' : 'text-muted-foreground'}" />
        <span
          class="min-w-0 flex-1 truncate whitespace-nowrap text-left transition-opacity duration-150 {open
            ? 'opacity-100 delay-75'
            : 'opacity-0'}"
        >
          {item.label}
        </span>
        {#if item.count !== undefined}
          <span
            class="text-xs tabular-nums text-muted-foreground transition-opacity duration-150 {open
              ? 'opacity-100'
              : 'opacity-0'}"
          >
            {item.count}
          </span>
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
</div>
