import {
  isTauri,
  kioskGetIcons,
  kioskGetProducts,
  kioskProcessActions,
  kioskSetAction,
  kioskStatus,
} from "./bridge";
import type { ActionRequest, KioskProduct, KioskStatus } from "./types";
import { toast } from "$lib/ui/toast.svelte";

export type Filter = "all" | "installed" | "available" | "updates";

const POLL_INTERVAL_MS = 5000;
const POLL_TIMEOUT_MS = 30 * 60 * 1000;
/** background refresh — picks up action requests set server-side (e.g. via AD groups) */
const AUTO_REFRESH_MS = 5 * 60 * 1000;

function isPending(p: KioskProduct): boolean {
  return p.actionRequest !== "" && p.actionRequest !== "none";
}

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
  /** pending productIds the user has already been notified about */
  private notifiedPending = new Set<string>();

  pending = $derived(this.products.filter(isPending));

  installed = $derived(this.products.filter((p) => p.installationStatus === "installed"));
  updatable = $derived(this.installed.filter((p) => p.updatePossible));

  filtered = $derived.by(() => {
    const q = this.search.trim().toLowerCase();
    let list = this.products;
    if (this.filter === "installed") list = this.installed;
    else if (this.filter === "available") list = list.filter((p) => p.installationStatus !== "installed");
    else if (this.filter === "updates") list = this.updatable;
    if (q) {
      list = list.filter(
        (p) =>
          p.productId.toLowerCase().includes(q) ||
          (p.productName ?? "").toLowerCase().includes(q) ||
          (p.description ?? "").toLowerCase().includes(q),
      );
    }
    // products with an available update first, rest alphabetical
    return [...list].sort((a, b) => {
      const ua = a.updatePossible ? 0 : 1;
      const ub = b.updatePossible ? 0 : 1;
      if (ua !== ub) return ua - ub;
      return (a.productName || a.productId).localeCompare(b.productName || b.productId, "de");
    });
  });

  async init() {
    if (!isTauri()) {
      this.loading = false;
      return;
    }
    try {
      this.status = await kioskStatus();
      await this.refresh();
      this.lastSync = new Date();
      // requests left over from a previous session (or set by an admin):
      // resume polling so their completion is reflected
      if (this.pending.length) void this.pollUntilDone();
      setInterval(() => void this.backgroundRefresh(), AUTO_REFRESH_MS);
    } catch (err) {
      toast.error("Verbindung zum opsi-Client-Dienst fehlgeschlagen", String(err));
    } finally {
      this.loading = false;
    }
  }

  /** Periodic refresh: surfaces software assigned server-side (e.g. through AD
   *  group membership → opsi action requests) without waiting for the next
   *  opsi event, and offers to run it right away. */
  private async backgroundRefresh() {
    if (this.processing || this.syncing) return;
    try {
      await this.refresh();
    } catch {
      return;
    }
    const fresh = this.pending.filter((p) => !this.notifiedPending.has(p.productId));
    if (fresh.length) {
      fresh.forEach((p) => this.notifiedPending.add(p.productId));
      toast.action(
        fresh.length === 1
          ? `Neue Zuweisung: ${fresh[0].productName || fresh[0].productId}`
          : `${fresh.length} neue Software-Zuweisungen`,
        { label: "Jetzt ausführen", onClick: () => void this.sync() },
        { description: "Diesem Computer wurde Software zugewiesen." },
      );
    }
  }

  /** Sync like Company Portal: refresh from the server, then execute every
   *  pending action request immediately. */
  async sync() {
    if (this.syncing) return;
    this.syncing = true;
    try {
      await this.refresh();
      if (this.pending.length && !this.processing) {
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

  async refresh() {
    this.products = await kioskGetProducts();
    try {
      this.icons = await kioskGetIcons(
        this.products.map((p) => p.productId),
        this.status?.depotId,
      );
    } catch {
      // icons are optional
    }
  }

  /** Set the action request and execute it right away. */
  async runAction(productId: string, action: Exclude<ActionRequest, "none">) {
    const name =
      this.products.find((p) => p.productId === productId)?.productName ?? productId;
    this.busy = new Set([...this.busy, productId]);
    try {
      await kioskSetAction(productId, action);
      await kioskProcessActions([productId]);
      await this.refresh();
      toast.success(
        action === "uninstall" ? `${name} wird deinstalliert` : `${name} wird installiert`,
        "Die Ausführung läuft im Hintergrund.",
      );
      void this.pollUntilDone();
    } catch (err) {
      // roll the request back so no orphaned action request lingers
      try {
        await kioskSetAction(productId, "none");
        await this.refresh();
      } catch {
        // best effort
      }
      toast.error("Aktion fehlgeschlagen", String(err));
    } finally {
      const next = new Set(this.busy);
      next.delete(productId);
      this.busy = next;
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
        "Die Ausführung läuft im Hintergrund.",
      );
      void this.pollUntilDone();
    } catch (err) {
      toast.error("Updates fehlgeschlagen", String(err));
    }
  }

  /** Clear a leftover action request without executing it. */
  async cancelAction(productId: string) {
    this.busy = new Set([...this.busy, productId]);
    try {
      await kioskSetAction(productId, "none");
      await this.refresh();
    } catch (err) {
      toast.error("Aktion konnte nicht abgebrochen werden", String(err));
    } finally {
      const next = new Set(this.busy);
      next.delete(productId);
      this.busy = next;
    }
  }

  private async pollUntilDone() {
    if (this.processing) return;
    this.processing = true;
    const started = Date.now();
    // remember exactly what is being processed so the completion toast can
    // name it instead of a generic "alle Aktionen abgeschlossen"
    const watched = new Map<string, string>();
    const capture = () => {
      for (const p of this.pending) {
        if (!watched.has(p.productId)) watched.set(p.productId, p.actionRequest);
      }
    };
    capture();

    const finish = () => {
      this.processing = false;
      const results = [...watched.entries()]
        .map(([id, action]) => ({
          action,
          product: this.products.find((p) => p.productId === id),
        }))
        .filter((r) => r.product);
      const failed = results.filter((r) => r.product!.actionResult === "failed");
      const ok = results.filter((r) => r.product!.actionResult !== "failed");

      if (failed.length) {
        toast.warning(
          "Abgeschlossen mit Fehlern",
          failed.map((r) => r.product!.productName || r.product!.productId).join(", "),
        );
      } else if (ok.length === 1) {
        const r = ok[0];
        const name = r.product!.productName || r.product!.productId;
        toast.success(r.action === "uninstall" ? `${name} deinstalliert` : `${name} installiert`);
      } else if (ok.length > 1) {
        toast.success(
          `${ok.length} Aktionen abgeschlossen`,
          ok.map((r) => r.product!.productName || r.product!.productId).join(", "),
        );
      }
    };

    const tick = async () => {
      try {
        await this.refresh();
      } catch {
        // transient — opsiclientd may be busy during action processing
      }
      capture();
      if (this.pending.length === 0) {
        finish();
        return;
      }
      if (Date.now() - started > POLL_TIMEOUT_MS) {
        this.processing = false;
        return;
      }
      setTimeout(tick, POLL_INTERVAL_MS);
    };
    setTimeout(tick, POLL_INTERVAL_MS);
  }
}

export const kiosk = new KioskStore();
