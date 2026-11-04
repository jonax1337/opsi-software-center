<script lang="ts" module>
  export type ConfirmRequest = {
    title: string;
    description: string;
    confirmLabel: string;
    onconfirm: () => void;
  };
</script>

<script lang="ts">
  import { Button, Dialog } from "$lib/ui";

  /** Asks before an action the user cannot take back (restart, admin scripts).
   *  Open while `request` is set; `onclose` clears it. */
  type Props = { request: ConfirmRequest | null; onclose: () => void };

  let { request, onclose }: Props = $props();

  function confirm() {
    const run = request?.onconfirm;
    onclose();
    run?.();
  }
</script>

<Dialog
  open={request !== null}
  onOpenChange={(open) => !open && onclose()}
  title={request?.title}
  description={request?.description}
>
  {#snippet footer()}
    <Button variant="outline" onclick={onclose}>Abbrechen</Button>
    <Button onclick={confirm}>{request?.confirmLabel}</Button>
  {/snippet}
</Dialog>
