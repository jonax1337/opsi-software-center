import { invoke } from "@tauri-apps/api/core";
import type { ActionRequest, KioskProduct, KioskStatus } from "./types";

export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export async function kioskStatus(): Promise<KioskStatus> {
  return invoke("kiosk_status");
}

export async function kioskGetProducts(): Promise<KioskProduct[]> {
  return invoke("kiosk_get_products");
}

export async function kioskSetAction(productId: string, action: ActionRequest): Promise<void> {
  return invoke("kiosk_set_action", { productId, action });
}

/** Omit productIds to process ALL pending action requests (sync). */
export async function kioskProcessActions(productIds?: string[]): Promise<void> {
  return invoke("kiosk_process_actions", { productIds: productIds ?? null });
}

/** productId → data-URL for products that have an icon file deployed locally
 *  or on the depot share */
export async function kioskGetIcons(
  productIds: string[],
  depotId?: string,
): Promise<Record<string, string>> {
  return invoke("kiosk_get_icons", { productIds, depotId: depotId ?? null });
}
