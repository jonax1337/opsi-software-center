<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { Minus, Square, Copy, X } from "@lucide/svelte";
  import appIcon from "../../assets/icon.png";
  import { onMount } from "svelte";
  import type { Snippet } from "svelte";

  type Props = {
    title?: string;
    children?: Snippet;
    actions?: Snippet;
  };
  let { title = "Software Center", children, actions }: Props = $props();

  const win = getCurrentWindow();
  let maximized = $state(false);

  onMount(() => {
    win.isMaximized().then((m) => (maximized = m));
    const un = win.onResized(async () => {
      maximized = await win.isMaximized();
    });
    return () => {
      un.then((u) => u());
    };
  });
</script>

<header
  data-tauri-drag-region
  class="titlebar select-none flex items-center h-9 shrink-0 bg-surface-chrome backdrop-blur-xl border-b border-hairline text-foreground"
>
  <div data-tauri-drag-region class="flex items-center pl-3 pr-1.5">
    <img src={appIcon} alt="" class="size-4 pointer-events-none" draggable="false" />
  </div>

  <div
    data-tauri-drag-region
    class="flex-1 flex items-center gap-3 text-xs font-medium text-muted-foreground tracking-tight"
  >
    {title}
    {@render children?.()}
  </div>

  {#if actions}
    <div class="flex items-center h-full pr-1" style="-webkit-app-region: no-drag">
      {@render actions()}
    </div>
  {/if}

  <div class="flex items-center h-full">
    <button type="button" class="titlebar-btn" aria-label="Minimieren" onclick={() => win.minimize()}>
      <Minus class="size-3.5" />
    </button>
    <button
      type="button"
      class="titlebar-btn"
      aria-label={maximized ? "Wiederherstellen" : "Maximieren"}
      onclick={() => win.toggleMaximize()}
    >
      {#if maximized}
        <Copy class="size-3 -scale-x-100" />
      {:else}
        <Square class="size-3" />
      {/if}
    </button>
    <button
      type="button"
      class="titlebar-btn titlebar-btn-close"
      aria-label="Schließen"
      onclick={() => win.close()}
    >
      <X class="size-4" />
    </button>
  </div>
</header>

<style>
  .titlebar {
    -webkit-app-region: drag;
  }
  .titlebar :global(button) {
    -webkit-app-region: no-drag;
  }
  :global(.titlebar-btn) {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 46px;
    height: 100%;
    color: var(--color-foreground);
    background: transparent;
    transition: background-color 120ms ease;
    cursor: pointer;
  }
  :global(.titlebar-btn:hover) {
    background-color: var(--color-muted);
  }
  :global(.titlebar-btn-close:hover) {
    background-color: oklch(0.6 0.235 27);
    color: white;
  }
</style>
