<script lang="ts">
  import { DataField, InfoBanner, SectionHeading } from "$lib/ui";
  import AppIcon from "./AppIcon.svelte";
  import ProductActions from "./ProductActions.svelte";
  import ProductStatus from "./ProductStatus.svelte";
  import { displayName, displayVersion } from "$lib/kiosk/product";
  import type { KioskProduct } from "$lib/kiosk/types";

  /** Back navigation lives in the title bar, as in Company Portal. */
  let { product }: { product: KioskProduct } = $props();

  const requirements = $derived(
    (product.requirements ?? []).map((r) => r.requiredProductId).filter(Boolean),
  );
</script>

<!-- Company Portal detail: big icon, then name, grey version line, status
     and the primary action directly below the name. -->
<div class="mb-8 flex items-start gap-6">
  <AppIcon productId={product.productId} size="lg" />
  <div class="min-w-0 flex-1">
    <h1 class="text-[28px] font-semibold leading-9">{displayName(product)}</h1>
    <div class="text-sm text-muted-foreground">{displayVersion(product)}</div>
    <div class="mt-1 empty:hidden"><ProductStatus {product} /></div>
    <div class="mt-4"><ProductActions {product} /></div>
  </div>
</div>

{#if product.actionResult === "failed"}
  <InfoBanner tone="error" class="mb-6">
    Die letzte Aktion für diese App ist fehlgeschlagen. Versuchen Sie es erneut oder wenden
    Sie sich an den IT-Support.
  </InfoBanner>
{/if}

<!-- product.advice is admin-only packaging info (visible in opsi-configed) — never shown to end users -->

<section class="mb-8 max-w-3xl">
  <SectionHeading title="Beschreibung" />
  <p class="whitespace-pre-line text-sm leading-relaxed">
    {product.description || "Für diese App ist keine Beschreibung hinterlegt."}
  </p>
</section>

<section>
  <SectionHeading title="Informationen" />
  <div class="grid max-w-3xl grid-cols-2 gap-x-8 gap-y-5 @xl:grid-cols-3">
    <DataField label="Verfügbare Version" value={product.versionStr} />
    <DataField label="Installierte Version" value={product.installedVerStr || "—"} />
    <DataField label="Produkt-ID" value={product.productId} />
    {#if requirements.length}
      <DataField label="Abhängigkeiten" value={requirements.join(", ")} />
    {/if}
  </div>
</section>
