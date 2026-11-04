import { getSystemInfo, isTauri, type SystemInfo } from "$lib/kiosk/bridge";

/** Device/account inventory shared by the Geräte and Hilfe pages. Queried once
 *  per app run: the PowerShell/CIM lookup takes seconds and rarely changes. */
class SystemInfoStore {
  info = $state<SystemInfo | null>(null);
  loading = $state(isTauri());
  #requested = false;

  load() {
    if (this.#requested || !isTauri()) return;
    this.#requested = true;
    getSystemInfo()
      .then((info) => (this.info = info))
      .catch(() => {
        // no inventory is shown as "nicht verfügbar" by the pages
      })
      .finally(() => (this.loading = false));
  }
}

export const systemInfo = new SystemInfoStore();
