<script lang="ts">
  import { onMount } from "svelte";
  import { Play, Spinner, Wrench } from "$lib/ui/icons";
  import { Button, InfoBanner, ListCard, ListRow, RowIcon, SectionHeading, toast } from "$lib/ui";
  import ConfirmDialog, { type ConfirmRequest } from "$lib/components/ConfirmDialog.svelte";
  import { adminActionRun, adminActionsList, isTauri, type AdminAction } from "$lib/kiosk/bridge";

  /** Maintenance scripts the IT ships with the opsi package. Hidden entirely
   *  when none are configured. */
  let actions = $state<AdminAction[]>([]);
  let loadError = $state<string | null>(null);
  let running = $state<string | null>(null);
  let confirmation = $state<ConfirmRequest | null>(null);

  onMount(async () => {
    if (!isTauri()) return;
    try {
      actions = await adminActionsList();
    } catch (err) {
      loadError = String(err);
    }
  });

  function start(action: AdminAction) {
    if (!action.confirm) {
      void run(action);
      return;
    }
    confirmation = {
      title: `${action.label}?`,
      description: action.confirm,
      confirmLabel: "Ausführen",
      onconfirm: () => run(action),
    };
  }

  async function run(action: AdminAction) {
    running = action.id;
    try {
      const message = await adminActionRun(action.id);
      toast.success(`${action.label}: erledigt`, message ?? undefined);
    } catch (err) {
      toast.error(`${action.label} fehlgeschlagen`, String(err));
    } finally {
      running = null;
    }
  }
</script>

{#if loadError}
  <InfoBanner tone="error" class="mt-8">
    Die Aktionen Ihrer IT konnten nicht geladen werden: {loadError}
  </InfoBanner>
{:else if actions.length}
  <SectionHeading title="Aktionen Ihrer IT" class="mt-8" />
  <ListCard>
    {#each actions as action (action.id)}
      <ListRow align="center">
        <RowIcon icon={Wrench} />
        <div class="min-w-0 flex-1">
          <p class="text-sm font-medium">{action.label}</p>
          {#if action.description}
            <p class="text-xs text-muted-foreground">{action.description}</p>
          {/if}
        </div>
        <Button variant="outline" size="sm" disabled={running !== null} onclick={() => start(action)}>
          {#if running === action.id}
            <Spinner class="size-4 animate-spin" />
            Wird ausgeführt…
          {:else}
            <Play class="size-4" />
            Ausführen
          {/if}
        </Button>
      </ListRow>
    {/each}
  </ListCard>
{/if}

<ConfirmDialog request={confirmation} onclose={() => (confirmation = null)} />
