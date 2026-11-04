<script lang="ts">
  import { onMount, untrack } from "svelte";
  import {
    Apps,
    Downloads,
    Home,
    Laptop,
    QuestionCircle,
    Settings,
  } from "$lib/ui/icons";
  import { NavPane, Toaster } from "$lib/ui";
  import type { NavItem } from "$lib/ui/NavPane.svelte";
  import CatalogGate from "$lib/components/CatalogGate.svelte";
  import ProductDetail from "$lib/components/ProductDetail.svelte";
  import TitleBar from "$lib/components/TitleBar.svelte";
  import AppsView from "$lib/views/AppsView.svelte";
  import DeviceView from "$lib/views/DeviceView.svelte";
  import DownloadsView from "$lib/views/DownloadsView.svelte";
  import HelpView from "$lib/views/HelpView.svelte";
  import HomeView from "$lib/views/HomeView.svelte";
  import SettingsView from "$lib/views/SettingsView.svelte";
  import { kiosk } from "$lib/kiosk/store.svelte";
  import type { Filter } from "$lib/kiosk/product";
  import { systemInfo } from "$lib/system/systemInfo.svelte";

  type View = "home" | "apps" | "downloads" | "devices" | "help" | "settings";
  /** One step of the title bar's back history. */
  type Location = { view: View; selectedId: string | null };

  const VIEW_LABELS: Record<View, string> = {
    home: "Startseite",
    apps: "Apps",
    downloads: "Downloads und Updates",
    devices: "Geräte",
    help: "Hilfe und Support",
    settings: "Einstellungen",
  };

  let view = $state<View>("home");
  let selectedId = $state<string | null>(null);
  let history = $state<Location[]>([]);
  // Compact icon rail by default, like Company Portal; the hamburger expands it.
  let navOpen = $state(false);

  const selected = $derived(
    selectedId ? (kiosk.products.find((p) => p.productId === selectedId) ?? null) : null,
  );
  const downloadsCount = $derived(kiosk.pending.length + kiosk.updatesToOffer.length);

  const navItems: NavItem[] = $derived([
    { value: "home", label: VIEW_LABELS.home, icon: Home },
    { value: "apps", label: VIEW_LABELS.apps, icon: Apps },
    {
      value: "downloads",
      label: VIEW_LABELS.downloads,
      icon: Downloads,
      count: downloadsCount || undefined,
    },
    { value: "devices", label: VIEW_LABELS.devices, icon: Laptop },
    { value: "help", label: VIEW_LABELS.help, icon: QuestionCircle },
  ]);
  const navFooter: NavItem[] = [{ value: "settings", label: VIEW_LABELS.settings, icon: Settings }];

  function go(next: Location) {
    if (next.view === view && next.selectedId === selectedId) return;
    history = [...history, { view, selectedId }];
    view = next.view;
    selectedId = next.selectedId;
  }

  function navigate(next: View) {
    go({ view: next, selectedId: null });
    kiosk.search = "";
  }

  function back() {
    const previous = history.at(-1);
    if (!previous) return;
    history = history.slice(0, -1);
    view = previous.view;
    selectedId = previous.selectedId;
  }

  function showAll(filter: Filter) {
    navigate("apps");
    kiosk.filter = filter;
  }

  function openProduct(productId: string) {
    go({ view, selectedId: productId });
  }

  // Typing in the title-bar search shows the results on the Apps page, like Company Portal.
  $effect(() => {
    if (!kiosk.search.trim()) return;
    untrack(() => {
      if (view !== "apps" || selectedId) go({ view: "apps", selectedId: null });
    });
  });

  // Retriggers the page transition on every navigation.
  const transitionKey = $derived(selected ? `detail:${selected.productId}` : view);

  onMount(() => {
    void kiosk.init();
    systemInfo.load();
  });
</script>

<div class="h-full flex flex-col">
  <TitleBar canGoBack={history.length > 0} onback={back} onprofile={() => navigate("help")} />

  <div class="flex-1 flex min-h-0">
    <NavPane
      items={navItems}
      footer={navFooter}
      value={view}
      onselect={(v) => navigate(v as View)}
      bind:open={navOpen}
    />

    <!-- Content layer: LayerOnMica with the WinUI rounded top-left corner; the
         Mica backdrop peeks through at the notch and behind the nav pane. -->
    <div
      class="relative flex-1 min-w-0 flex flex-col rounded-tl-lg border-l border-t border-hairline bg-background/55 overflow-hidden"
    >
      <main class="flex-1 overflow-y-auto">
        <!-- Left-aligned like a WinUI NavigationView page, not centered. -->
        <div class="max-w-6xl px-10 pt-8 pb-12">
          {#key transitionKey}
            <div class={selected ? "animate-drill" : "animate-fade"}>
              {#if selected}
                <ProductDetail product={selected} />
              {:else if view === "devices"}
                <DeviceView />
              {:else if view === "help"}
                <HelpView />
              {:else if view === "settings"}
                <SettingsView />
              {:else}
                <CatalogGate>
                  {#if view === "home"}
                    <HomeView onopen={openProduct} onshowall={showAll} onnavigate={navigate} />
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
