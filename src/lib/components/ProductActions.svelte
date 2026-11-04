<script lang="ts">
  import { ArrowUpCircle, Download, Loader2, Trash2, X } from "@lucide/svelte";
  import { Button } from "$lib/ui";
  import { kiosk } from "$lib/kiosk/store.svelte";
  import { hasUpdate, isInstalled, productState } from "$lib/kiosk/product";
  import type { KioskProduct } from "$lib/kiosk/types";

  /** The install/update/uninstall buttons of the detail page. */
  let { product }: { product: KioskProduct } = $props();

  const id = $derived(product.productId);
  const state = $derived(productState(product, kiosk.processing));
  const busy = $derived(kiosk.busy.has(id));
</script>

<div class="flex shrink-0 flex-wrap items-center gap-2">
  {#if busy}
    <Button disabled>
      <Loader2 class="size-4 animate-spin" />
      Bitte warten…
    </Button>
  {:else if state === "running"}
    <Button disabled>
      <Loader2 class="size-4 animate-spin" />
      Wird ausgeführt…
    </Button>
  {:else if state === "pending"}
    <Button variant="outline" onclick={() => kiosk.cancelAction(id)}>
      <X class="size-4" />
      Abbrechen
    </Button>
  {:else if isInstalled(product)}
    {#if hasUpdate(product)}
      <Button disabled={kiosk.processing} onclick={() => kiosk.runAction(id, "setup")}>
        <ArrowUpCircle class="size-4" />
        Aktualisieren
      </Button>
    {/if}
    {#if product.hasUninstall}
      <Button
        variant="outline"
        class="text-destructive hover:text-destructive"
        disabled={kiosk.processing}
        onclick={() => kiosk.runAction(id, "uninstall")}
      >
        <Trash2 class="size-4" />
        Deinstallieren
      </Button>
    {/if}
  {:else if product.hasSetup}
    <Button disabled={kiosk.processing} onclick={() => kiosk.runAction(id, "setup")}>
      <Download class="size-4" />
      Installieren
    </Button>
  {/if}
</div>
