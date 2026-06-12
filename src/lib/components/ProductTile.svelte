<script lang="ts">
  import { Package, CircleCheck, ArrowUpCircle, Clock, Loader2 } from "@lucide/svelte";
  import { kiosk } from "$lib/kiosk/store.svelte";
  import type { KioskProduct } from "$lib/kiosk/types";

  type Props = { product: KioskProduct; onclick: () => void };
  let { product, onclick }: Props = $props();

  const installed = $derived(product.installationStatus === "installed");
  const pending = $derived(product.actionRequest !== "" && product.actionRequest !== "none");
  const running = $derived(pending && kiosk.processing);
  const icon = $derived(kiosk.icons[product.productId]);
</script>

<button
  type="button"
  {onclick}
  class="flex flex-col items-start gap-3 p-4 rounded-xl border border-hairline-strong bg-card/95 backdrop-blur-md card-inset text-left transition-all duration-200 hover:-translate-y-0.5 hover:shadow-md cursor-pointer outline-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px]"
>
  <div
    class="grid place-items-center size-14 rounded-lg bg-surface-3 border border-hairline overflow-hidden"
  >
    {#if icon}
      <img src={icon} alt="" class="size-9 object-contain" />
    {:else}
      <Package class="size-7 text-muted-foreground" />
    {/if}
  </div>

  <div class="min-w-0 w-full">
    <div class="text-sm font-medium truncate">{product.productName || product.productId}</div>
    <div class="mt-1 flex items-center gap-1.5 text-xs text-muted-foreground">
      {#if running}
        <Loader2 class="size-3.5 animate-spin shrink-0" />
        <span class="truncate">
          {product.actionRequest === "uninstall" ? "Wird deinstalliert…" : "Wird installiert…"}
        </span>
      {:else if pending}
        <Clock class="size-3.5 shrink-0" />
        <span class="truncate">Aktion ausstehend</span>
      {:else if installed && product.updatePossible}
        <ArrowUpCircle class="size-3.5 text-amber-600 dark:text-amber-400 shrink-0" />
        <span class="truncate">Update verfügbar</span>
      {:else if installed}
        <CircleCheck class="size-3.5 text-success shrink-0" />
        <span class="truncate">Installiert</span>
      {:else}
        <span class="truncate">{product.versionStr}</span>
      {/if}
    </div>
  </div>
</button>
