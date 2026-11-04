/**
 * Pure product rules shared by the store and the views: naming, status,
 * filtering and sorting. No Svelte state here, so everything is unit-testable.
 */
import type { KioskProduct } from "./types";

export type Filter = "all" | "installed" | "available" | "updates";

/** What the UI shows for a product, in priority order (first match wins). */
export type ProductState = "running" | "pending" | "update" | "installed" | "available";

const SORT_LOCALE = "de";

export function displayName(p: KioskProduct): string {
  return p.productName || p.productId;
}

/** An action request is set and not yet processed by opsi. */
export function isPending(p: KioskProduct): boolean {
  return p.actionRequest !== "" && p.actionRequest !== "none";
}

export function isInstalled(p: KioskProduct): boolean {
  return p.installationStatus === "installed";
}

export function hasUpdate(p: KioskProduct): boolean {
  return isInstalled(p) && p.updatePossible;
}

/** The version the user has (installed) or would get (available). */
export function displayVersion(p: KioskProduct): string {
  return isInstalled(p) ? p.installedVerStr : p.versionStr;
}

/** `processing` = the store's poll loop is running, i.e. opsi works on requests. */
export function productState(p: KioskProduct, processing: boolean): ProductState {
  if (isPending(p)) return processing ? "running" : "pending";
  if (hasUpdate(p)) return "update";
  if (isInstalled(p)) return "installed";
  return "available";
}

export function matchesFilter(p: KioskProduct, filter: Filter): boolean {
  switch (filter) {
    case "all":
      return true;
    case "installed":
      return isInstalled(p);
    case "available":
      return !isInstalled(p);
    case "updates":
      return hasUpdate(p);
  }
}

export function matchesSearch(p: KioskProduct, query: string): boolean {
  const q = query.trim().toLowerCase();
  if (!q) return true;
  return [p.productId, p.productName, p.description].some((field) =>
    (field ?? "").toLowerCase().includes(q),
  );
}

/** Products with an available update first, the rest alphabetical. */
export function compareProducts(a: KioskProduct, b: KioskProduct): number {
  const byUpdate = Number(!a.updatePossible) - Number(!b.updatePossible);
  if (byUpdate !== 0) return byUpdate;
  return displayName(a).localeCompare(displayName(b), SORT_LOCALE);
}

export function selectProducts(
  products: KioskProduct[],
  filter: Filter,
  query: string,
): KioskProduct[] {
  return products
    .filter((p) => matchesFilter(p, filter) && matchesSearch(p, query))
    .sort(compareProducts);
}

/** "Firefox" for one product, "Firefox, VLC" for several. */
export function joinNames(products: KioskProduct[]): string {
  return products.map(displayName).join(", ");
}
