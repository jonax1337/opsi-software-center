<script lang="ts">
  import type { Component } from "svelte";
  import { ArrowClockwise, DocumentText, Globe, Spinner } from "$lib/ui/icons";
  import { Button, toast } from "$lib/ui";
  import { deviceFlushDns, deviceGpresultReport, deviceGpupdate, isTauri } from "$lib/kiosk/bridge";

  /** Self-help buttons a supporter typically asks for on the phone. */
  type Tool = { id: string; label: string; icon: Component; run: () => Promise<void> };

  const TOOLS: Tool[] = [
    { id: "gpupdate", label: "Richtlinien aktualisieren", icon: ArrowClockwise, run: refreshPolicies },
    { id: "dns", label: "DNS-Cache leeren", icon: Globe, run: flushDns },
    { id: "gpresult", label: "Richtlinienbericht", icon: DocumentText, run: openPolicyReport },
  ];

  let running = $state<string | null>(null);

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

  async function openPolicyReport() {
    await deviceGpresultReport();
    toast.success("Richtlinienbericht erstellt", "Wird im Browser geöffnet.");
  }
</script>

<div class="flex flex-wrap gap-2">
  {#each TOOLS as tool (tool.id)}
    <Button
      variant="outline"
      size="sm"
      disabled={!isTauri() || running !== null}
      onclick={() => runTool(tool)}
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
