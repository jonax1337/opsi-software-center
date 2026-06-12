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
};

export type ActionRequest = "setup" | "uninstall" | "none";
