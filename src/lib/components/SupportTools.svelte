<script lang="ts">
  import type { Component } from "svelte";
  import {
    ArrowClockwise,
    Broom,
    Desktop,
    DocumentText,
    FolderZip,
    Globe,
    Power,
    Spinner,
  } from "$lib/ui/icons";
  import { Button, toast } from "$lib/ui";
  import ConfirmDialog, { type ConfirmRequest } from "$lib/components/ConfirmDialog.svelte";
  import { formatBytes } from "$lib/format";
  import { kiosk } from "$lib/kiosk/store.svelte";
  import {
    deviceCleanTemp,
    deviceDiagnosticsBundle,
    deviceFlushDns,
    deviceGpresultReport,
    deviceGpupdate,
    deviceRestart,
    deviceRestartExplorer,
    isTauri,
  } from "$lib/kiosk/bridge";

  /** Self-help buttons a supporter typically asks for on the phone. Tools
   *  with a `confirm` text ask first because they interrupt the user. */
  type Tool = {
    id: string;
    label: string;
    icon: Component;
    confirm?: Omit<ConfirmRequest, "onconfirm">;
    run: () => Promise<void>;
  };

  const TOOLS: Tool[] = [
    { id: "gpupdate", label: "Richtlinien aktualisieren", icon: ArrowClockwise, run: refreshPolicies },
    { id: "dns", label: "DNS-Cache leeren", icon: Globe, run: flushDns },
    { id: "temp", label: "Temporäre Dateien löschen", icon: Broom, run: cleanTemp },
    {
      id: "explorer",
      label: "Explorer neu starten",
      icon: Desktop,
      confirm: {
        title: "Explorer neu starten?",
        description:
          "Taskleiste und Startmenü verschwinden kurz, geöffnete Explorer-Fenster werden geschlossen. Andere Programme laufen weiter.",
        confirmLabel: "Neu starten",
      },
      run: restartExplorer,
    },
    { id: "gpresult", label: "Richtlinienbericht", icon: DocumentText, run: openPolicyReport },
    { id: "diagnostics", label: "Diagnosepaket erstellen", icon: FolderZip, run: createDiagnostics },
    {
      id: "restart",
      label: "Computer neu starten",
      icon: Power,
      confirm: {
        title: "Computer jetzt neu starten?",
        description: "Speichern Sie vorher alle geöffneten Dokumente. Der Neustart beginnt sofort.",
        confirmLabel: "Jetzt neu starten",
      },
      run: deviceRestart,
    },
  ];

  let running = $state<string | null>(null);
  let confirmation = $state<ConfirmRequest | null>(null);

  function start(tool: Tool) {
    if (tool.confirm) {
      confirmation = { ...tool.confirm, onconfirm: () => runTool(tool) };
    } else {
      void runTool(tool);
    }
  }

  async function runTool(tool: Tool) {
    running = tool.id;
    try {
      await tool.run();
    } catch (err) {
      toast.error(`${tool.label} fehlgeschlagen`, String(err));
    } finally {
      running = null;
    }
  }

  async function refreshPolicies() {
    const r = await deviceGpupdate(true);
    if (r.computer && r.user) {
      toast.success("Richtlinien aktualisiert", "Computer und Benutzer erfolgreich.");
    } else {
      toast.warning(
        "Teilweise aktualisiert",
        `Computer: ${r.computer ? "OK" : "Fehler"}, Benutzer: ${r.user ? "OK" : "Fehler"}`,
      );
    }
  }

  async function flushDns() {
    if (await deviceFlushDns()) toast.success("DNS-Cache geleert");
    else toast.error("DNS-Cache leeren fehlgeschlagen");
  }

  async function cleanTemp() {
    const r = await deviceCleanTemp();
    if (r.files === 0) {
      toast.success("Nichts zu löschen", "Dateien in Verwendung oder von heute bleiben erhalten.");
    } else {
      const files = r.files === 1 ? "1 Datei" : `${r.files.toLocaleString("de-DE")} Dateien`;
      toast.success(`${formatBytes(r.bytes)} freigegeben`, `${files} gelöscht.`);
    }
  }

  async function restartExplorer() {
    await deviceRestartExplorer();
    toast.success("Explorer neu gestartet");
  }

  async function openPolicyReport() {
    await deviceGpresultReport();
    toast.success("Richtlinienbericht erstellt", "Wird im Browser geöffnet.");
  }

  async function createDiagnostics() {
    const path = await deviceDiagnosticsBundle();
    const file = path.split("\\").pop() ?? path;
    toast.success("Diagnosepaket auf dem Desktop gespeichert", `${file} – bitte an Ihr Ticket anhängen.`);
  }
</script>

<div class="flex flex-wrap gap-2">
  {#each TOOLS as tool (tool.id)}
    <Button
      variant={tool.id === "restart" && kiosk.rebootPending ? "default" : "outline"}
      size="sm"
      disabled={!isTauri() || running !== null}
      onclick={() => start(tool)}
    >
      {#if running === tool.id}
        <Spinner class="size-4 animate-spin" />
      {:else}
        <tool.icon class="size-4" />
      {/if}
      {tool.label}
    </Button>
  {/each}
</div>

<ConfirmDialog request={confirmation} onclose={() => (confirmation = null)} />
