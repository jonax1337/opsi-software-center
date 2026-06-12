<script lang="ts">
  import { cn } from "$lib/utils";

  type Props = {
    value: number;
    size?: "sm" | "md";
    tone?: "primary" | "success" | "warning" | "destructive";
    class?: string;
  };

  let { value, size = "sm", tone = "primary", class: klass }: Props = $props();

  const pct = $derived(Math.max(0, Math.min(100, value)));
  const heightCls = $derived({ sm: "h-1", md: "h-2" }[size]);
  const toneCls = $derived(
    {
      primary: "bg-primary",
      success: "bg-success",
      warning: "bg-warning",
      destructive: "bg-destructive",
    }[tone],
  );
</script>

<div class={cn("rounded-full bg-muted overflow-hidden", heightCls, klass)}>
  <div
    class={cn("h-full rounded-full transition-all duration-500", toneCls)}
    style="width: {pct}%"
  ></div>
</div>
