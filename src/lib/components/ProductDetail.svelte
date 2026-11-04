<script lang="ts">
  import { ArrowLeft } from "@lucide/svelte";
  import { DataField, InfoBanner, SectionHeading } from "$lib/ui";
  import AppIcon from "./AppIcon.svelte";
  import ProductActions from "./ProductActions.svelte";
  import ProductStatus from "./ProductStatus.svelte";
  import { displayName, isInstalled } from "$lib/kiosk/product";
  import type { KioskProduct } from "$lib/kiosk/types";

  type Props = { product: KioskProduct; backLabel: string; onback: () => void };
  let { product, backLabel, onback }: Props = $props();

  const requirements = $derived(
    (product.requirements ?? []).map((r) => r.requiredProductId).filter(Boolean),
  );
</script>

<button
  type="button"
  onclick={onback}
  class="mb-6 inline-flex items-center gap-1.5 text-sm text-muted-foreground transition-colors hover:text-foreground cursor-pointer"
>
  <ArrowLeft class="size-4" />
  {backLabel}
</button>

<!-- Company Portal detail: big icon, name + status, primary action on the right -->
<div class="mb-8 flex flex-wrap items-center gap-5">
  <AppIcon productId={product.productId} size="lg" />
  <div class="min-w-0 flex-1">
    <h1 class="truncate text-2xl font-semibold leading-tight">{displayName(product)}</h1>
    <div class="mt-1 font-mono text-sm tabular-nums text-muted-foreground">
      {isInstalled(product) ? product.installedVerStr : product.versionStr}
    </div>
    <div class="mt-2">
      <ProductStatus {product} />
    </div>
  </div>
  <ProductActions {product} />
</div>

{#if product.actionResult === "failed"}
  <InfoBanner tone="error" class="mb-6">
    Die letzte Aktion für diese App ist fehlgeschlagen. Versuchen Sie es erneut oder wenden
    Sie sich an den IT-Support.
  </InfoBanner>
{/if}

<!-- product.advice is admin-only packaging info (visible in opsi-configed) — never shown to end users -->

<div class="grid gap-8 md:grid-cols-[minmax(0,1fr)_15rem]">
  <section>
    <SectionHeading title="Beschreibung" />
    <p class="whitespace-pre-line text-sm leading-relaxed text-muted-foreground">
      {product.description || "Für diese App ist keine Beschreibung hinterlegt."}
    </p>
  </section>

  <aside class="space-y-5">
    <SectionHeading title="Informationen" />
    <DataField label="Verfügbare Version" value={product.versionStr} mono />
    <DataField label="Installierte Version" value={product.installedVerStr || "—"} mono />
    <DataField label="Produkt-ID" value={product.productId} mono />
    {#if requirements.length}
      <DataField label="Abhängigkeiten" value={requirements.join(", ")} />
    {/if}
  </aside>
</div>
