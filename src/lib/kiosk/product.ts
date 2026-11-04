/**
 * Pure product rules shared by the store and the views: naming, status,
 * filtering and sorting. No Svelte state here, so everything is unit-testable.
 */
import type { Catalog, KioskProduct } from "./types";

export type Filter = "all" | "installed" | "available" | "updates";

/** Company Portal "Sortieren nach" options. */
export type Sort = "name-asc" | "name-desc" | "publisher-asc" | "publisher-desc";

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

export function publisherOf(p: KioskProduct, catalog: Catalog): string {
  return catalog[p.productId]?.publisher ?? "";
}

export function categoryOf(p: KioskProduct, catalog: Catalog): string {
  return catalog[p.productId]?.category ?? "";
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

/** Like Company Portal: by name or publisher (plus id and description). */
export function matchesSearch(p: KioskProduct, query: string, catalog: Catalog = {}): boolean {
  const q = query.trim().toLowerCase();
  if (!q) return true;
  return [p.productId, p.productName, p.description, publisherOf(p, catalog)].some((field) =>
    (field ?? "").toLowerCase().includes(q),
  );
}

/** "" = every category. */
export function matchesCategory(p: KioskProduct, category: string, catalog: Catalog): boolean {
  return !category || categoryOf(p, catalog) === category;
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
  catalog: Catalog = {},
): KioskProduct[] {
  return products
    .filter((p) => matchesFilter(p, filter) && matchesSearch(p, query, catalog))
    .sort(compareProducts);
}

const byName = (a: KioskProduct, b: KioskProduct) =>
  displayName(a).localeCompare(displayName(b), SORT_LOCALE);

/** New array in the chosen order. Products without a publisher sort last in
 *  both publisher orders; ties fall back to the name. */
export function sortProducts(products: KioskProduct[], sort: Sort, catalog: Catalog): KioskProduct[] {
  const byPublisher = (a: KioskProduct, b: KioskProduct) => {
    const [pa, pb] = [publisherOf(a, catalog), publisherOf(b, catalog)];
    if (!pa !== !pb) return pa ? -1 : 1;
    const order = pa.localeCompare(pb, SORT_LOCALE);
    return (sort === "publisher-desc" ? -order : order) || byName(a, b);
  };
  const compare: Record<Sort, (a: KioskProduct, b: KioskProduct) => number> = {
    "name-asc": byName,
    "name-desc": (a, b) => byName(b, a),
    "publisher-asc": byPublisher,
    "publisher-desc": byPublisher,
  };
  return [...products].sort(compare[sort]);
}

export type CategorySummary = { name: string; products: KioskProduct[] };

/** Categories that have at least one product, alphabetical. */
export function categoriesOf(products: KioskProduct[], catalog: Catalog): CategorySummary[] {
  const groups = new Map<string, KioskProduct[]>();
  for (const p of products) {
    const name = categoryOf(p, catalog);
    if (name) groups.set(name, [...(groups.get(name) ?? []), p]);
  }
  return [...groups.entries()]
    .map(([name, members]) => ({ name, products: members.sort(byName) }))
    .sort((a, b) => a.name.localeCompare(b.name, SORT_LOCALE));
}

/** Company Portal "Ausgewählte Apps": flagged in the catalog, alphabetical. */
export function featuredProducts(products: KioskProduct[], catalog: Catalog): KioskProduct[] {
  return products.filter((p) => catalog[p.productId]?.featured).sort(byName);
}

/** "Firefox" for one product, "Firefox, VLC" for several. */
export function joinNames(products: KioskProduct[]): string {
  return products.map(displayName).join(", ");
}
