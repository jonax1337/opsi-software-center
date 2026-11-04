/** Product record as returned by opsi's getKioskProductInfosForClient
 *  (opsiconfd/backend/rpc/ext_kiosk.py). */
export type KioskProduct = {
  productId: string;
  productName?: string;
  description?: string;
  advice?: string;
  priority?: number;
  productType?: string;
  productVersion: string;
  packageVersion: string;
  versionStr: string;
  installedVerStr: string;
  installedProdVer: string;
  installedPackVer: string;
  updatePossible: boolean;
  possibleAction: "" | "setup" | "uninstall";
  installationStatus: "" | "installed" | "not_installed" | "unknown";
  actionRequest: "" | "none" | "setup" | "uninstall" | "update" | "always" | "once" | "custom";
  actionResult: "" | "none" | "failed" | "successful";
  hasSetup?: boolean;
  hasUninstall?: boolean;
  requirements?: {
    requiredProductId: string;
    required: string;
    preRequired: string;
    postRequired: string;
  }[];
};

export type KioskStatus = {
  connected: boolean;
  mock: boolean;
  clientId: string;
  depotId: string;
  version: string;
  /** Why opsiclientd is unreachable; empty when connected or in demo mode. */
  error: string;
};

export type ActionRequest = "setup" | "uninstall" | "none";

/** Optional per-product metadata from the admin-deployed `catalog.json`
 *  (src-tauri/src/catalog.rs) — the kiosk API has no publisher or category. */
export type CatalogEntry = {
  publisher: string;
  category: string;
  featured: boolean;
};

/** productId → entry; products without an entry are simply not listed. */
export type Catalog = Record<string, CatalogEntry>;
