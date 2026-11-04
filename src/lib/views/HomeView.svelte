<script lang="ts">
  import { onMount } from "svelte";
  import {
    Alert,
    ArrowCircleUp,
    ArrowClockwise,
    CheckmarkCircle,
    ChevronRight,
    Clock,
    ErrorCircle,
    Spinner,
  } from "$lib/ui/icons";
  import { Button, EmptyState, PageHeader, SectionHeading } from "$lib/ui";
  import ProductCollection from "$lib/components/ProductCollection.svelte";
  import ProductTile from "$lib/components/ProductTile.svelte";
  import { CONNECTION_LABEL, kiosk } from "$lib/kiosk/store.svelte";
  import { displayName, featuredProducts, selectProducts, type Filter } from "$lib/kiosk/product";
  import { systemInfo } from "$lib/system/systemInfo.svelte";
  import { formatTime } from "$lib/format";

  /** Company Portal home: the featured-apps banner on top, then app cards on
   *  the left and notifications plus the "Ihre Geräte" card on the right. */
  type Props = {
    onopen: (productId: string) => void;
    onshowall: (filter: Filter) => void;
    onnavigate: (view: "downloads" | "devices") => void;
  };
  let { onopen, onshowall, onnavigate }: Props = $props();

  /** Cards per home section — "Alle anzeigen" leads to the rest. */
  const PREVIEW_COUNT = 6;
  const FEATURED_COUNT = 4;

  onMount(() => systemInfo.load());

  const available = $derived(selectProducts(kiosk.products, "available", ""));
  const installed = $derived(selectProducts(kiosk.products, "installed", ""));
  const featured = $derived(featuredProducts(kiosk.products, kiosk.branding.featuredProducts));
  const hostname = $derived(systemInfo.info?.pc?.hostname ?? "Dieser Computer");
  const deviceOk = $derived(kiosk.connection === "connected" || kiosk.connection === "demo");
</script>

<PageHeader title="Startseite" />

{#if featured.length}
  <!-- Company Portal "Featured Apps" banner: dark gradient, copy on the left,
       app cards on the right. Wide: one row of four cards that shrink rather
       than wrap (a 3 + 1 orphan looks broken); narrow: copy on top, 2 × 2 cards. -->
  <section class="featured-banner @container/banner mb-8 overflow-hidden rounded-lg p-6 text-white">
    <!-- A container query styles descendants only, so the row/column switch
         lives on this inner wrapper, not on the section itself. -->
    <div class="flex flex-col gap-6 @3xl/banner:flex-row @3xl/banner:items-center">
      <div class="@3xl/banner:w-56 @3xl/banner:shrink-0">
        <h2 class="text-xl font-semibold">Ausgewählte Apps</h2>
        <p class="mt-1 text-sm text-white/80">
          Von Ihrer IT empfohlen — damit Sie direkt loslegen können.
        </p>
        {#if featured.length > FEATURED_COUNT}
          <Button class="mt-4" variant="outline" size="sm" onclick={() => onshowall("all")}>
            Alle anzeigen
          </Button>
        {/if}
      </div>
      <div class="grid min-w-0 flex-1 grid-cols-2 gap-3 text-foreground @3xl/banner:grid-cols-4">
        {#each featured.slice(0, FEATURED_COUNT) as product (product.productId)}
          <ProductTile {product} onclick={() => onopen(product.productId)} />
        {/each}
      </div>
    </div>
  </section>
{/if}

<div class="grid gap-x-8 gap-y-6 @4xl:grid-cols-[minmax(0,1fr)_17rem]">
  <div class="min-w-0">
    {#if kiosk.updatable.length}
      <section class="mb-8">
        <SectionHeading title="Updates verfügbar">
          {#snippet actions()}
            <Button size="sm" disabled={kiosk.processing} onclick={() => kiosk.updateAll()}>
              <ArrowCircleUp class="size-4" />
              Alle aktualisieren
            </Button>
          {/snippet}
        </SectionHeading>
        <ProductCollection products={kiosk.updatable.slice(0, PREVIEW_COUNT)} {onopen} />
      </section>
    {/if}

    {#snippet section(heading: string, filter: Filter, products: typeof available)}
      {#if products.length}
        <section class="mb-8">
          <SectionHeading title={heading}>
            {#snippet actions()}
              {#if products.length > PREVIEW_COUNT}
                <Button variant="link" size="sm" onclick={() => onshowall(filter)}>
                  Alle anzeigen
                </Button>
              {/if}
            {/snippet}
          </SectionHeading>
          <ProductCollection products={products.slice(0, PREVIEW_COUNT)} {onopen} />
        </section>
      {/if}
    {/snippet}

    {@render section("Verfügbare Apps", "available", available)}
    {@render section("Installierte Apps", "installed", installed)}

    {#if kiosk.products.length === 0}
      <EmptyState>Für diesen Computer ist noch keine Software freigegeben.</EmptyState>
    {/if}
  </div>

  <aside class="min-w-0 space-y-8">
    <section>
      <h2 class="mb-3 flex items-center gap-2 text-base font-semibold text-primary">
        <Alert class="size-4" /> Benachrichtigungen
      </h2>
      <div class="card-inset divide-y divide-hairline overflow-hidden rounded-lg bg-card text-sm">
        {#each kiosk.pending as p (p.productId)}
          <button
            type="button"
            onclick={() => onopen(p.productId)}
            class="flex w-full items-center gap-3 px-4 py-3 text-left transition-colors hover:bg-surface-3 cursor-pointer"
          >
            {#if kiosk.processing}
              <Spinner class="size-4 shrink-0 animate-spin text-primary" />
              <span class="min-w-0 truncate">
                {p.actionRequest === "uninstall" ? "Wird deinstalliert" : "Wird installiert"}: {displayName(p)}
              </span>
            {:else}
              <Clock class="size-4 shrink-0 text-muted-foreground" />
              <span class="min-w-0 truncate">Ausstehend: {displayName(p)}</span>
            {/if}
          </button>
        {/each}
        {#if kiosk.updatesToOffer.length}
          <button
            type="button"
            onclick={() => onnavigate("downloads")}
            class="flex w-full items-center gap-3 px-4 py-3 text-left transition-colors hover:bg-surface-3 cursor-pointer"
          >
            <ArrowCircleUp class="size-4 shrink-0 text-amber-600 dark:text-amber-400" />
            <span class="min-w-0 flex-1 truncate">
              {kiosk.updatesToOffer.length === 1
                ? "1 Update verfügbar"
                : `${kiosk.updatesToOffer.length} Updates verfügbar`}
            </span>
            <ChevronRight class="size-4 shrink-0 text-muted-foreground" />
          </button>
        {/if}
        {#if kiosk.rebootPending}
          <div class="flex items-center gap-3 px-4 py-3">
            <ArrowClockwise class="size-4 shrink-0 text-amber-600 dark:text-amber-400" />
            <span class="min-w-0">Neustart ausstehend</span>
          </div>
        {/if}
        {#if !kiosk.pending.length && !kiosk.updatesToOffer.length && !kiosk.rebootPending}
          <p class="px-4 py-3 text-muted-foreground">Keine neuen Benachrichtigungen.</p>
        {/if}
      </div>
    </section>

    <section>
      <h2 class="mb-3 text-xl font-semibold">Ihre Geräte</h2>
      <button
        type="button"
        onclick={() => onnavigate("devices")}
        class="card-inset w-full rounded-lg bg-card p-5 text-left transition-colors hover:bg-surface-3 cursor-pointer outline-none focus-visible:ring-2 focus-visible:ring-ring"
      >
        <span class="inline-block rounded-sm bg-primary px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-primary-foreground">
          Dieses Gerät
        </span>
        <span class="mt-4 block truncate text-lg font-semibold">{hostname}</span>
        <span class="block text-xs text-muted-foreground">
          {kiosk.lastSync
            ? `Zuletzt synchronisiert: ${formatTime(kiosk.lastSync)} Uhr`
            : "Noch nicht synchronisiert"}
        </span>
        <span class="mt-4 flex items-center gap-2 text-xs">
          {#if deviceOk}
            <CheckmarkCircle class="size-4 text-success" />
          {:else}
            <ErrorCircle class="size-4 text-destructive" />
          {/if}
          {CONNECTION_LABEL[kiosk.connection]}
        </span>
      </button>
    </section>
  </aside>
</div>


<style>
  .featured-banner {
    background: linear-gradient(
      100deg,
      #111 0%,
      color-mix(in oklch, var(--titlebar) 70%, #111) 45%,
      color-mix(in oklch, var(--titlebar) 35%, #ddd) 100%
    );
  }
</style>
