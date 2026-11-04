<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { Download, House, LayoutGrid, LifeBuoy, Search, Settings } from "@lucide/svelte";
  import { NavPane, SearchInput, Toaster } from "$lib/ui";
  import type { NavItem } from "$lib/ui/NavPane.svelte";
  import CatalogGate from "$lib/components/CatalogGate.svelte";
  import ProductDetail from "$lib/components/ProductDetail.svelte";
  import AppsView from "$lib/views/AppsView.svelte";
  import DownloadsView from "$lib/views/DownloadsView.svelte";
  import HelpView from "$lib/views/HelpView.svelte";
  import HomeView from "$lib/views/HomeView.svelte";
  import SettingsView from "$lib/views/SettingsView.svelte";
  import { kiosk } from "$lib/kiosk/store.svelte";
  import type { Filter } from "$lib/kiosk/product";
  import brandMark from "./assets/mark.svg";

  type View = "home" | "apps" | "downloads" | "help" | "settings";

  const VIEW_LABELS: Record<View, string> = {
    home: "Startseite",
    apps: "Apps",
    downloads: "Downloads & Updates",
    help: "Hilfe & Support",
    settings: "Einstellungen",
  };

  let view = $state<View>("home");
  let selectedId = $state<string | null>(null);
  let navOpen = $state(true);

  const selected = $derived(
    selectedId ? (kiosk.products.find((p) => p.productId === selectedId) ?? null) : null,
  );
  const downloadsCount = $derived(kiosk.pending.length + kiosk.updatesToOffer.length);

  const navItems: NavItem[] = $derived([
    { value: "home", label: VIEW_LABELS.home, icon: House },
    { value: "apps", label: VIEW_LABELS.apps, icon: LayoutGrid },
    {
      value: "downloads",
      label: VIEW_LABELS.downloads,
      icon: Download,
      count: downloadsCount || undefined,
    },
  ]);
  const navFooter: NavItem[] = [
    { value: "help", label: VIEW_LABELS.help, icon: LifeBuoy },
    { value: "settings", label: VIEW_LABELS.settings, icon: Settings },
  ];

  function navigate(next: View) {
    view = next;
    selectedId = null;
    kiosk.search = "";
  }

  function showAll(filter: Filter) {
    kiosk.filter = filter;
    navigate("apps");
  }

  function openProduct(productId: string) {
    selectedId = productId;
  }

  // Typing in the nav search shows the results on the Apps page, like Company Portal.
  $effect(() => {
    if (!kiosk.search.trim()) return;
    untrack(() => {
      view = "apps";
      selectedId = null;
    });
  });

  // Retriggers the page transition on every navigation.
  const transitionKey = $derived(selected ? `detail:${selected.productId}` : view);

  onMount(() => {
    void kiosk.init();
  });
</script>

<div class="h-full flex flex-col">
  <div class="flex-1 flex min-h-0">
    <NavPane
      items={navItems}
      footer={navFooter}
      value={view}
      onselect={(v) => navigate(v as View)}
      bind:open={navOpen}
    >
      {#snippet header(open)}
        {#if open}
          <div class="mb-3 flex items-center gap-2.5 px-1">
            <img src={brandMark} alt="" class="size-6 shrink-0" />
            <span class="truncate text-sm font-semibold">Software Center</span>
          </div>
          <SearchInput bind:value={kiosk.search} placeholder="Apps suchen" />
        {:else}
          <button
            type="button"
            aria-label="Apps suchen"
            title="Apps suchen"
            onclick={() => (navOpen = true)}
            class="grid h-10 w-9 place-items-center rounded-sm hover:bg-surface-3 transition-colors cursor-pointer outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            <Search class="size-5 text-muted-foreground" />
          </button>
        {/if}
      {/snippet}
    </NavPane>

    <!-- Content layer: LayerOnMica with the WinUI rounded top-left corner; the
         Mica backdrop peeks through at the notch and behind the nav pane. -->
    <div
      class="relative flex-1 min-w-0 flex flex-col rounded-tl-lg border-l border-t border-hairline bg-background/55 overflow-hidden"
    >
      <main class="flex-1 overflow-y-auto">
        <div class="max-w-5xl mx-auto px-8 pt-7 pb-12">
          {#key transitionKey}
            <div class={selected ? "animate-drill" : "animate-fade"}>
              {#if selected}
                <ProductDetail
                  product={selected}
                  backLabel={VIEW_LABELS[view]}
                  onback={() => (selectedId = null)}
                />
              {:else if view === "help"}
                <HelpView />
              {:else if view === "settings"}
                <SettingsView />
              {:else}
                <CatalogGate>
                  {#if view === "home"}
                    <HomeView onopen={openProduct} onshowall={showAll} />
                  {:else if view === "apps"}
                    <AppsView onopen={openProduct} />
                  {:else}
                    <DownloadsView onopen={openProduct} />
                  {/if}
                </CatalogGate>
              {/if}
            </div>
          {/key}
        </div>
      </main>
    </div>
  </div>

  <Toaster />
</div>
