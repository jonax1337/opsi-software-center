<script lang="ts">
  import { onMount } from "svelte";
  import { ArrowLeft, Person, Search } from "$lib/ui/icons";
  import { isTauri } from "$lib/kiosk/bridge";
  import { kiosk } from "$lib/kiosk/store.svelte";
  import { systemInfo } from "$lib/system/systemInfo.svelte";
  import { windowControls } from "$lib/system/desktop";
  import { initials } from "$lib/format";

  /** Company Portal title bar: accent-colored, back button, app + company name,
   *  centered app search, account avatar and the window's caption buttons
   *  (the native frame is off — `decorations: false`). */
  type Props = {
    canGoBack: boolean;
    onback: () => void;
    onprofile: () => void;
  };
  let { canGoBack, onback, onprofile }: Props = $props();

  /** Segoe Fluent Icons glyphs — the same ones the native caption buttons use. */
  const GLYPH = { minimize: "", maximize: "", restore: "", close: "" };

  let maximized = $state(false);

  onMount(() => {
    const sync = () => void windowControls.isMaximized().then((m) => (maximized = m));
    sync();
    const unlisten = windowControls.onResized(sync);
    return () => void unlisten.then((stop) => stop());
  });

  const user = $derived(systemInfo.info?.user);
  const avatar = $derived(user ? initials(user.displayName, user.username) : "");
  const companyName = $derived(kiosk.branding.companyName);
</script>

<header
  data-tauri-drag-region
  class="flex h-12 shrink-0 select-none items-center bg-titlebar text-titlebar-foreground"
>
  <!-- Same 40×36 rounded box, offset and hover as the nav rail's buttons below. -->
  <button
    type="button"
    aria-label="Zurück"
    title="Zurück"
    disabled={!canGoBack}
    onclick={onback}
    class="mx-1.5 grid h-9 w-10 shrink-0 place-items-center rounded-sm transition-colors hover:bg-white/10 active:bg-white/5 disabled:opacity-40 disabled:hover:bg-transparent outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-white"
  >
    <ArrowLeft class="size-4" />
  </button>

  <div data-tauri-drag-region class="flex min-w-0 shrink items-center gap-2.5 pr-4">
    <span data-tauri-drag-region class="truncate text-xs">Software Portal</span>
    {#if companyName}
      <span data-tauri-drag-region class="h-4 w-px shrink-0 bg-white/30"></span>
      <span data-tauri-drag-region class="truncate text-xs font-semibold uppercase tracking-wide">
        {companyName}
      </span>
    {/if}
  </div>

  <div data-tauri-drag-region class="flex min-w-0 flex-1 justify-center px-2">
    <!-- Fluent TextBox: slightly translucent at rest, solid when focused, and an
         accent underline that grows from the center (clipped by the rounded box). -->
    <label class="search-box group relative w-full max-w-xl overflow-hidden rounded-sm">
      <span class="sr-only">Apps suchen</span>
      <Search
        class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground transition-colors duration-150 group-focus-within:text-primary"
      />
      <input
        type="search"
        bind:value={kiosk.search}
        placeholder="Apps suchen"
        class="h-8 w-full bg-card/85 pl-9 pr-3 text-sm text-foreground transition-colors duration-150 placeholder:text-muted-foreground outline-none hover:bg-card/95 focus:bg-card"
      />
      <span
        aria-hidden="true"
        class="search-underline pointer-events-none absolute inset-x-0 bottom-0 h-0.5 origin-center scale-x-0 bg-primary group-focus-within:scale-x-100"
      ></span>
    </label>
  </div>

  <button
    type="button"
    onclick={onprofile}
    title={user?.displayName || user?.username || "Mein Konto"}
    aria-label="Mein Konto"
    class="mx-2 grid size-8 shrink-0 place-items-center rounded-full bg-white/90 text-xs font-semibold text-titlebar transition-colors hover:bg-white outline-none focus-visible:ring-2 focus-visible:ring-white"
  >
    {#if avatar}{avatar}{:else}<Person class="size-4" />{/if}
  </button>

  {#if isTauri()}
    {#snippet caption(label: string, glyph: string, action: () => Promise<void>, danger = false)}
      <button
        type="button"
        aria-label={label}
        title={label}
        onclick={() => void action()}
        class="caption-glyph grid h-full w-[46px] shrink-0 place-items-center text-[10px] transition-colors outline-none {danger
          ? 'hover:bg-[#c42b1c] hover:text-white active:bg-[#c42b1c]/90'
          : 'hover:bg-white/10 active:bg-white/5'}"
      >
        {glyph}
      </button>
    {/snippet}
    <div class="flex h-full shrink-0 self-start">
      {@render caption("Minimieren", GLYPH.minimize, windowControls.minimize)}
      {@render caption(
        maximized ? "Wiederherstellen" : "Maximieren",
        maximized ? GLYPH.restore : GLYPH.maximize,
        windowControls.toggleMaximize,
      )}
      {@render caption("Schließen", GLYPH.close, windowControls.close, true)}
    </div>
  {/if}
</header>

<style>
  .caption-glyph {
    font-family: "Segoe Fluent Icons", "Segoe MDL2 Assets", sans-serif;
  }

  /* Grows in like the nav rail's selection pill (Fluent decelerate); collapses faster. */
  .search-underline {
    transition: transform 150ms var(--ease-fluent-accel);
  }
  .search-box:focus-within > .search-underline {
    transition: transform 250ms var(--ease-fluent-decel);
  }
</style>
