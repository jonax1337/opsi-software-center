<script lang="ts">
  import { onMount } from "svelte";
  import {
    ArrowUpCircle,
    Download,
    Info,
    LayoutGrid,
    PackageCheck,
    SearchX,
  } from "@lucide/svelte";
  import {
    Button,
    EmptyState,
    InfoBanner,
    NavPane,
    PageHeader,
    SearchInput,
    Toaster,
  } from "$lib/ui";
  import type { NavItem } from "$lib/ui/NavPane.svelte";
  import ProductTile from "$lib/components/ProductTile.svelte";
  import ProductDetail from "$lib/components/ProductDetail.svelte";
  import InfoView from "$lib/components/InfoView.svelte";
  import { kiosk, type Filter } from "$lib/kiosk/store.svelte";
  import { isTauri } from "$lib/kiosk/bridge";

  let selectedId = $state<string | null>(null);
  let view = $state<"apps" | "info">("apps");
  let navOpen = $state(true);

  const selected = $derived(
    selectedId ? (kiosk.products.find((p) => p.productId === selectedId) ?? null) : null,
  );

  function filterCount(f: Filter): number {
    if (f === "all") return kiosk.products.length;
    if (f === "installed") return kiosk.installed.length;
    if (f === "updates") return kiosk.updatable.length;
    return kiosk.products.length - kiosk.installed.length;
  }

  const navItems: NavItem[] = $derived([
    { value: "available", label: "Verfügbar", icon: Download, count: filterCount("available") },
    { value: "all", label: "Alle Apps", icon: LayoutGrid, count: filterCount("all") },
    { value: "installed", label: "Installiert", icon: PackageCheck, count: filterCount("installed") },
    { value: "updates", label: "Updates", icon: ArrowUpCircle, count: filterCount("updates") },
  ]);
  const navFooter: NavItem[] = [{ value: "info", label: "Info", icon: Info }];
  const navValue = $derived(view === "info" ? "info" : kiosk.filter);

  function selectNav(v: string) {
    selectedId = null;
    if (v === "info") {
      view = "info";
      return;
    }
    view = "apps";
    kiosk.filter = v as Filter;
  }

  const PAGE: Record<Filter, { title: string; desc: string }> = {
    available: {
      title: "Verfügbar",
      desc: "Freigegebene Software für diesen Computer installieren.",
    },
    all: { title: "Alle Apps", desc: "Alle für diesen Computer freigegebenen Apps." },
    installed: {
      title: "Installiert",
      desc: "Auf diesem Computer installierte Apps verwalten oder entfernen.",
    },
    updates: { title: "Updates", desc: "Verfügbare Aktualisierungen für installierte Apps." },
  };
  const page = $derived(PAGE[kiosk.filter]);

  // Key that retriggers the page transition on every navigation.
  const transitionKey = $derived(
    selected ? `detail:${selected.productId}` : `${view}:${kiosk.filter}`,
  );

  onMount(() => {
    void kiosk.init();
  });
</script>

<div class="h-full flex flex-col">
  <div class="flex-1 flex min-h-0">
    <NavPane
      items={navItems}
      footer={navFooter}
      value={navValue}
      onselect={selectNav}
      bind:open={navOpen}
    />

    <!-- Content layer: LayerOnMica with the WinUI rounded top-left corner; the
         Mica backdrop peeks through at the notch and behind the nav pane. -->
    <div
      class="relative flex-1 min-w-0 flex flex-col rounded-tl-lg border-l border-t border-hairline bg-background/55 overflow-hidden"
    >
      <main class="flex-1 overflow-y-auto">
        <div class="max-w-4xl mx-auto px-8 pt-7 pb-12">
          {#key transitionKey}
            <div class={selected ? "animate-drill" : "animate-fade"}>
              {#if selected}
                <ProductDetail product={selected} onback={() => (selectedId = null)} />
              {:else if view === "info"}
                <InfoView />
              {:else}
                <PageHeader title={page.title} description={page.desc}>
                  {#snippet actions()}
                    {#if kiosk.updatable.length > 0}
                      <Button size="sm" disabled={kiosk.processing} onclick={() => kiosk.updateAll()}>
                        <ArrowUpCircle class="size-4" />
                        {kiosk.updatable.length === 1
                          ? "1 Update installieren"
                          : `${kiosk.updatable.length} Updates installieren`}
                      </Button>
                    {/if}
                  {/snippet}
                </PageHeader>

                {#if !isTauri()}
                  <InfoBanner tone="info" class="mb-6">
                    Browser-Vorschau — der opsi-Client-Dienst ist nur in der Desktop-App erreichbar.
                  </InfoBanner>
                {:else if kiosk.status && !kiosk.status.connected && !kiosk.status.mock}
                  <InfoBanner tone="warning" class="mb-6">
                    Keine Verbindung zum opsi-Client-Dienst (opsiclientd). Läuft der Dienst auf
                    diesem Computer?
                  </InfoBanner>
                {/if}

                <div class="mb-6">
                  <SearchInput bind:value={kiosk.search} placeholder="Apps suchen…" class="w-full" />
                </div>

                {#if kiosk.loading}
                  <EmptyState loading>Apps werden geladen…</EmptyState>
                {:else if kiosk.filtered.length === 0}
                  <EmptyState icon={SearchX}>
                    {kiosk.products.length === 0
                      ? "Für diesen Computer ist keine Software-on-Demand-Gruppe freigegeben."
                      : "Keine Apps für diesen Filter gefunden."}
                  </EmptyState>
                {:else}
                  <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-3 stagger">
                    {#each kiosk.filtered as product (product.productId)}
                      <ProductTile {product} onclick={() => (selectedId = product.productId)} />
                    {/each}
                  </div>
                {/if}
              {/if}
            </div>
          {/key}
        </div>
      </main>
    </div>
  </div>

  <Toaster />
</div>
