<script lang="ts">
  import { ArrowUpCircle, CircleCheck, Clock, Loader2 } from "@lucide/svelte";
  import { kiosk } from "$lib/kiosk/store.svelte";
  import { productState } from "$lib/kiosk/product";
  import type { KioskProduct } from "$lib/kiosk/types";

  /** One-line install status (icon + label), as under Company Portal tiles. */
  let { product }: { product: KioskProduct } = $props();

  const state = $derived(productState(product, kiosk.processing));
  const runningLabel = $derived(
    product.actionRequest === "uninstall" ? "Wird deinstalliert…" : "Wird installiert…",
  );
</script>

<span class="inline-flex min-w-0 items-center gap-1.5 text-xs text-muted-foreground">
  {#if state === "running"}
    <Loader2 class="size-3.5 shrink-0 animate-spin text-primary" />
    <span class="truncate">{runningLabel}</span>
  {:else if state === "pending"}
    <Clock class="size-3.5 shrink-0" />
    <span class="truncate">Ausstehend</span>
  {:else if state === "update"}
    <ArrowUpCircle class="size-3.5 shrink-0 text-amber-600 dark:text-amber-400" />
    <span class="truncate">Update verfügbar</span>
  {:else if state === "installed"}
    <CircleCheck class="size-3.5 shrink-0 text-success" />
    <span class="truncate">Installiert</span>
  {:else}
    <span class="truncate font-mono tabular-nums">{product.versionStr}</span>
  {/if}
</span>
