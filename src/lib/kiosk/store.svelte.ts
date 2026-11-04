import {
  deviceGpupdate,
  devicePendingReboot,
  getBranding,
  isTauri,
  kioskGetIcons,
  kioskGetProducts,
  kioskProcessActions,
  kioskSetAction,
  kioskStatus,
  NO_BRANDING,
  type Branding,
} from "./bridge";
import {
  displayName,
  hasUpdate,
  isInstalled,
  isPending,
  joinNames,
  selectProducts,
  type Filter,
} from "./product";
import type { ActionRequest, KioskProduct, KioskStatus } from "./types";
import { toast } from "$lib/ui/toast.svelte";
import {
  ensureAutostartDefault,
  initNotifications,
  notify,
  onTrayCheckUpdates,
} from "$lib/system/desktop";

export type { Filter };

const POLL_INTERVAL_MS = 5000;
const POLL_TIMEOUT_MS = 30 * 60 * 1000;
/** background refresh — picks up action requests set server-side (e.g. via AD
 *  groups) and reconnects when opsiclientd was not up yet at login */
const AUTO_REFRESH_MS = 5 * 60 * 1000;
const RUNS_IN_BACKGROUND = "Die Ausführung läuft im Hintergrund.";
const ASSIGNED_TO_DEVICE = "Diesem Computer wurde Software zugewiesen.";

class KioskStore {
  status = $state<KioskStatus | null>(null);
  products = $state<KioskProduct[]>([]);
  loading = $state(true);
  /** productIds whose action request is currently being written */
  busy = $state<Set<string>>(new Set());
  /** true while opsi processes fired action requests (poll loop running) */
  processing = $state(false);
  search = $state("");
  filter = $state<Filter>("all");
  /** productId → data-URL for locally deployed icon files */
  icons = $state<Record<string, string>>({});
  syncing = $state(false);
  lastSync = $state<Date | null>(null);
  /** Windows reboot pending (opsi's own prompt is often suppressed) */
  rebootPending = $state(false);
  branding = $state<Branding>(NO_BRANDING);
  /** pending productIds the user has already been notified about */
  private notifiedPending = new Set<string>();
  /** updatable productIds already surfaced as a native notification this session */
  private notifiedUpdates = new Set<string>();

  /** Live opsiclientd or the demo catalog — i.e. there is something to show. */
  connected = $derived(!!this.status && (this.status.connected || this.status.mock));
  pending = $derived(this.products.filter(isPending));
  installed = $derived(this.products.filter(isInstalled));
  updatable = $derived(this.products.filter(hasUpdate));
  /** Updates not already queued — what "Downloads und Updates" still offers. */
  updatesToOffer = $derived(this.updatable.filter((p) => !isPending(p)));
  filtered = $derived(selectProducts(this.products, this.filter, this.search));

  async init() {
    if (!isTauri()) {
      this.loading = false;
      return;
    }
    // Desktop integration: launch at login by default, ask for notification
    // permission, and let the tray menu trigger a sync.
    void ensureAutostartDefault();
    void initNotifications();
    void onTrayCheckUpdates(() => void this.sync());
    void this.loadBranding();
    setInterval(() => void this.backgroundRefresh(), AUTO_REFRESH_MS);
    await this.connect();
  }

  /** Retry after "Keine Verbindung" — opsiclientd may have started meanwhile. */
  async retry() {
    this.loading = true;
    await this.connect();
  }

  private async connect() {
    try {
      this.status = await kioskStatus();
      if (!this.connected) return;
      await this.refresh();
      this.lastSync = new Date();
      // requests left over from a previous session (or set by an admin):
      // resume polling so their completion is reflected
      if (this.pending.length) void this.pollUntilDone();
      this.notifyNewUpdates();
    } catch (err) {
      toast.error("Apps konnten nicht geladen werden", String(err));
    } finally {
      this.loading = false;
    }
  }

  /** Also called periodically: the opsi package may rewrite the branding
   *  while the app keeps running in the tray. */
  async loadBranding() {
    try {
      this.branding = await getBranding();
    } catch {
      // branding is optional — neutral texts are shown without it
    }
  }

  /** Periodic refresh: reconnects if needed, surfaces software assigned
   *  server-side without waiting for the next opsi event, and announces new
   *  updates — the window is usually hidden in the tray meanwhile. */
  private async backgroundRefresh() {
    void this.loadBranding();
    if (this.processing || this.syncing) return;
    if (!this.connected) {
      await this.connect();
      return;
    }
    try {
      await this.refresh();
    } catch {
      return; // transient; the next tick retries
    }
    this.announceNewAssignments();
    this.notifyNewUpdates();
  }

  private announceNewAssignments() {
    const fresh = this.pending.filter((p) => !this.notifiedPending.has(p.productId));
    if (!fresh.length) return;
    fresh.forEach((p) => this.notifiedPending.add(p.productId));
    const label =
      fresh.length === 1
        ? `Neue Zuweisung: ${displayName(fresh[0])}`
        : `${fresh.length} neue Software-Zuweisungen`;
    toast.action(
      label,
      { label: "Jetzt ausführen", onClick: () => void this.sync() },
      { description: ASSIGNED_TO_DEVICE },
    );
    void notify(label, ASSIGNED_TO_DEVICE);
  }

  /** One native notification for updates not yet announced this session. */
  private notifyNewUpdates() {
    const fresh = this.updatable.filter((p) => !this.notifiedUpdates.has(p.productId));
    if (!fresh.length) return;
    fresh.forEach((p) => this.notifiedUpdates.add(p.productId));
    const names = joinNames(fresh);
    if (fresh.length === 1) {
      void notify("Update verfügbar", `Für ${names} steht eine Aktualisierung bereit.`);
    } else {
      void notify(`${fresh.length} Updates verfügbar`, `Aktualisierungen bereit: ${names}`);
    }
  }

  /** Sync like Company Portal: refresh from the server, then execute every
   *  pending action request immediately. A group-policy refresh runs in
   *  parallel and reports back without blocking the sync spinner. */
  async sync() {
    if (this.syncing) return;
    this.syncing = true;
    if (this.status?.connected) void this.gpupdate();
    try {
      // a reconnect may already have started observing leftover requests —
      // they still have to be executed
      const reconnected = !this.connected;
      if (reconnected) await this.connect();
      await this.refresh();
      if (this.pending.length && (!this.processing || reconnected)) {
        await kioskProcessActions();
        void this.pollUntilDone();
      }
      this.lastSync = new Date();
    } catch (err) {
      toast.error("Synchronisierung fehlgeschlagen", String(err));
    } finally {
      this.syncing = false;
    }
  }

  /** gpupdate for both scopes; only failures surface as a toast. */
  private async gpupdate() {
    try {
      const r = await deviceGpupdate();
      const failed = [!r.computer && "Computer", !r.user && "Benutzer"].filter(Boolean);
      if (failed.length) {
        toast.error(
          "Gruppenrichtlinien-Aktualisierung fehlgeschlagen",
          `Bereich: ${failed.join(", ")}`,
        );
      }
    } catch {
      // gpupdate is best-effort — the opsi sync result matters more
    }
    void this.refreshRebootPending();
  }

  private async refreshRebootPending() {
    try {
      this.rebootPending = await devicePendingReboot();
    } catch {
      this.rebootPending = false;
    }
  }

  async refresh() {
    this.products = await kioskGetProducts();
    try {
      this.icons = await kioskGetIcons(
        this.products.map((p) => p.productId),
        this.status?.depotId,
      );
    } catch {
      // icons are optional — tiles fall back to a generic package symbol
    }
    void this.refreshRebootPending();
  }

  /** Set the action request and execute it right away. */
  async runAction(productId: string, action: Exclude<ActionRequest, "none">) {
    const product = this.products.find((p) => p.productId === productId);
    const name = product ? displayName(product) : productId;
    await this.withBusy(productId, async () => {
      try {
        await kioskSetAction(productId, action);
        await kioskProcessActions([productId]);
        await this.refresh();
        toast.success(
          action === "uninstall" ? `${name} wird deinstalliert` : `${name} wird installiert`,
          RUNS_IN_BACKGROUND,
        );
        void this.pollUntilDone();
      } catch (err) {
        await this.rollBack(productId);
        toast.error("Aktion fehlgeschlagen", String(err));
      }
    });
  }

  /** Clears the request again so no orphaned action request lingers. */
  private async rollBack(productId: string) {
    try {
      await kioskSetAction(productId, "none");
      await this.refresh();
    } catch {
      // the original error is already being reported
    }
  }

  /** Queue a setup for every product with an available update and run them. */
  async updateAll() {
    const targets = this.updatable.map((p) => p.productId);
    if (!targets.length || this.processing) return;
    try {
      for (const id of targets) {
        await kioskSetAction(id, "setup");
      }
      await kioskProcessActions(targets);
      await this.refresh();
      toast.success(
        targets.length === 1 ? "1 Update gestartet" : `${targets.length} Updates gestartet`,
        RUNS_IN_BACKGROUND,
      );
      void this.pollUntilDone();
    } catch (err) {
      toast.error("Updates fehlgeschlagen", String(err));
    }
  }

  /** Clear a leftover action request without executing it. */
  async cancelAction(productId: string) {
    await this.withBusy(productId, async () => {
      try {
        await kioskSetAction(productId, "none");
        await this.refresh();
      } catch (err) {
        toast.error("Aktion konnte nicht abgebrochen werden", String(err));
      }
    });
  }

  private async withBusy(productId: string, work: () => Promise<void>) {
    this.busy = new Set([...this.busy, productId]);
    try {
      await work();
    } finally {
      const next = new Set(this.busy);
      next.delete(productId);
      this.busy = next;
    }
  }

  /** Re-poll until opsi has processed every request, then report what
   *  happened by name (never a generic "alle Aktionen abgeschlossen"). */
  private async pollUntilDone() {
    if (this.processing) return;
    this.processing = true;
    const started = Date.now();
    /** productId → the action that was requested for it */
    const watched = new Map<string, string>();
    const watchPending = () => {
      for (const p of this.pending) {
        if (!watched.has(p.productId)) watched.set(p.productId, p.actionRequest);
      }
    };
    watchPending();

    const tick = async () => {
      try {
        await this.refresh();
      } catch {
        // transient — opsiclientd may be busy during action processing
      }
      watchPending();
      if (this.pending.length === 0) {
        this.processing = false;
        this.announceCompletion(watched);
      } else if (Date.now() - started > POLL_TIMEOUT_MS) {
        this.processing = false;
      } else {
        setTimeout(tick, POLL_INTERVAL_MS);
      }
    };
    setTimeout(tick, POLL_INTERVAL_MS);
  }

  private announceCompletion(watched: Map<string, string>) {
    const done = [...watched.entries()].flatMap(([id, action]) => {
      const product = this.products.find((p) => p.productId === id);
      return product ? [{ product, action }] : [];
    });
    const failed = done.filter((d) => d.product.actionResult === "failed").map((d) => d.product);
    const ok = done.filter((d) => d.product.actionResult !== "failed");

    if (failed.length) {
      toast.warning("Abgeschlossen mit Fehlern", joinNames(failed));
    } else if (ok.length === 1) {
      const name = displayName(ok[0].product);
      toast.success(ok[0].action === "uninstall" ? `${name} deinstalliert` : `${name} installiert`);
    } else if (ok.length > 1) {
      toast.success(`${ok.length} Aktionen abgeschlossen`, joinNames(ok.map((d) => d.product)));
    }
  }
}

export const kiosk = new KioskStore();
