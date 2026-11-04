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

export interface GpUpdateResult {
  computer: boolean;
  user: boolean;
}

/** Group-policy refresh for both scopes. `force` re-applies all policies. */
export async function deviceGpupdate(force = false): Promise<GpUpdateResult> {
  return invoke("device_gpupdate", { force });
}

/** Flush the DNS resolver cache (ipconfig /flushdns). */
export async function deviceFlushDns(): Promise<boolean> {
  return invoke("device_flush_dns");
}

/** Generate an HTML RSOP report (user scope) and open it; returns its path. */
export async function deviceGpresultReport(): Promise<string> {
  return invoke("device_gpresult_report");
}

/** Windows pending-reboot flag (CBS / Windows Update / pending file renames). */
export async function devicePendingReboot(): Promise<boolean> {
  return invoke("device_pending_reboot");
}

/** Shades of the user's Windows accent color (lightest→darkest), `#rrggbb`.
 *  Fields are null when the accent can't be read (e.g. browser preview). */
export interface AccentColor {
  base: string | null;
  light1: string | null;
  light2: string | null;
  light3: string | null;
  dark1: string | null;
  dark2: string | null;
  dark3: string | null;
}

/** Native Windows accent palette (HKCU). Used to override --primary/--ring. */
export async function getAccentColor(): Promise<AccentColor> {
  return invoke("get_accent_color");
}

export interface PcInfo {
  hostname: string;
  domain: string;
  manufacturer: string;
  model: string;
  serial: string;
  os: string;
  build: string;
  cpu: string;
  ramGb: number;
  diskFreeGb: number;
  diskTotalGb: number;
  ip: string | null;
  lastBoot: string; // ISO 8601
}

export interface UserInfo {
  displayName: string;
  username: string;
  mail: string;
  department: string;
  office: string;
  title: string;
  passwordExpires: string | null; // ISO 8601, null = never expires
}

export interface DiagInfo {
  logonServer: string | null;
  lastGpo: string | null; // ISO 8601 — last Group Policy processing
  lastUpdate: string | null; // ISO 8601 — newest installed hotfix
  lastUpdateId: string | null; // e.g. "KB5034123"
}

export interface SystemInfo {
  pc: PcInfo | null;
  user: UserInfo | null;
  diag: DiagInfo | null;
}

/** Device hardware facts (CIM/WMI) + the signed-in user's AD attributes
 *  (ADSISearcher). `user` is null off-domain. Standard-user-safe. */
export async function getSystemInfo(): Promise<SystemInfo> {
  return invoke("get_system_info");
}

/** Company name, IT-support contact and featured apps from the opsi package's
 *  product properties (`HKLM\SOFTWARE\opsi-kiosk\Branding`). Empty = not set. */
export interface Branding {
  companyName: string;
  supportPhone: string;
  supportEmail: string;
  supportUrl: string;
  /** product ids for the "Ausgewählte Apps" banner, in banner order */
  featuredProducts: string[];
}

export const NO_BRANDING: Branding = {
  companyName: "",
  supportPhone: "",
  supportEmail: "",
  supportUrl: "",
  featuredProducts: [],
};

export async function getBranding(): Promise<Branding> {
  return invoke("get_branding");
}

export type SupportChannel = "email" | "phone" | "website";

/** Open the configured support contact (mail client, phone app, browser). */
export async function openSupportContact(channel: SupportChannel): Promise<void> {
  return invoke("open_support_contact", { channel });
}

/** productId → data-URL for products that have an icon file deployed locally
 *  or on the depot share */
export async function kioskGetIcons(
  productIds: string[],
  depotId?: string,
): Promise<Record<string, string>> {
  return invoke("kiosk_get_icons", { productIds, depotId: depotId ?? null });
}
