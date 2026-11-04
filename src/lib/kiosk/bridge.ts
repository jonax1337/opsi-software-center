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

/** Restart the computer now (apps with unsaved work can still veto it). */
export async function deviceRestart(): Promise<void> {
  return invoke("device_restart");
}

/** Restart Windows Explorer (taskbar, start menu) in the user's session. */
export async function deviceRestartExplorer(): Promise<void> {
  return invoke("device_restart_explorer");
}

export interface CleanupResult {
  files: number;
  bytes: number;
}

/** Delete the user's temp files untouched for a day; files in use are skipped. */
export async function deviceCleanTemp(): Promise<CleanupResult> {
  return invoke("device_clean_temp");
}

/** ZIP with systeminfo, ipconfig, gpresult and readable opsi logs on the
 *  user's desktop; returns its path. */
export async function deviceDiagnosticsBundle(): Promise<string> {
  return invoke("device_diagnostics_bundle");
}

/** A maintenance script the admin ships via the opsi package
 *  (`%ProgramFiles%\Software Portal\actions\actions.json`). */
export interface AdminAction {
  id: string;
  label: string;
  description: string;
  /** confirmation question; empty = run on click */
  confirm: string;
}

export async function adminActionsList(): Promise<AdminAction[]> {
  return invoke("admin_actions_list");
}

/** Runs the script as the signed-in user; resolves with its last output line. */
export async function adminActionRun(id: string): Promise<string | null> {
  return invoke("admin_action_run", { id });
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
  /** company color `#rrggbb` for accents and title bar; "" = default blue */
  brandColor: string;
}

export const NO_BRANDING: Branding = {
  companyName: "",
  supportPhone: "",
  supportEmail: "",
  supportUrl: "",
  featuredProducts: [],
  brandColor: "",
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
