/**
 * Desktop-integration helpers: autostart, native notifications, the tray
 * "check for updates" event and the custom title bar's window controls. All no-ops in the browser preview (isTauri false),
 * so callers don't need to guard every call.
 */
import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { isTauri } from "$lib/kiosk/bridge";

/** Mirrors `tray::CHECK_UPDATES_EVENT` in the Rust backend. */
const CHECK_UPDATES_EVENT = "tray://check-updates";

/** Whether the app launches automatically at login. */
export async function autostartEnabled(): Promise<boolean> {
  if (!isTauri()) return false;
  try {
    return await isEnabled();
  } catch {
    return false;
  }
}

export async function setAutostart(on: boolean): Promise<boolean> {
  if (!isTauri()) return false;
  try {
    if (on) await enable();
    else await disable();
    return await isEnabled();
  } catch {
    return await autostartEnabled();
  }
}

/**
 * Turn autostart on by default the first time the app runs, but never fight a
 * later manual choice — once the user has toggled it we leave it alone.
 */
export async function ensureAutostartDefault(): Promise<void> {
  if (!isTauri()) return;
  if (localStorage.getItem("autostart-initialized")) return;
  await setAutostart(true);
  localStorage.setItem("autostart-initialized", "1");
}

let notificationsAllowed = false;

/** Ask for notification permission once, up front. */
export async function initNotifications(): Promise<void> {
  if (!isTauri()) return;
  try {
    notificationsAllowed = await isPermissionGranted();
    if (!notificationsAllowed) {
      notificationsAllowed = (await requestPermission()) === "granted";
    }
  } catch {
    notificationsAllowed = false;
  }
}

/** Fire a native OS notification (best effort — stays silent if not allowed). */
export async function notify(title: string, body: string): Promise<void> {
  if (!isTauri() || !notificationsAllowed) return;
  try {
    sendNotification({ title, body });
  } catch {
    // ignore — a missing notification is never worth surfacing an error
  }
}

/** Subscribe to the tray "Nach Updates suchen" menu entry. */
export async function onTrayCheckUpdates(cb: () => void): Promise<UnlistenFn> {
  if (!isTauri()) return () => {};
  return listen(CHECK_UPDATES_EVENT, () => cb());
}

/** Caption-button actions for the custom title bar (the window is undecorated).
 *  `close` only hides — the backend's CloseRequested handler keeps the app in the tray. */
export const windowControls = {
  minimize: () => (isTauri() ? getCurrentWindow().minimize() : Promise.resolve()),
  toggleMaximize: () => (isTauri() ? getCurrentWindow().toggleMaximize() : Promise.resolve()),
  close: () => (isTauri() ? getCurrentWindow().close() : Promise.resolve()),
  isMaximized: () => (isTauri() ? getCurrentWindow().isMaximized() : Promise.resolve(false)),
  /** Fires on every resize, incl. maximize/restore. */
  onResized: async (cb: () => void): Promise<UnlistenFn> =>
    isTauri() ? getCurrentWindow().onResized(() => cb()) : () => {},
};
