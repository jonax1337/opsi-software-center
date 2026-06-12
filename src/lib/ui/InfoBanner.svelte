<script lang="ts">
  import type { Component, Snippet } from "svelte";
  import { AlertTriangle, Info } from "@lucide/svelte";
  import { cn } from "$lib/utils";

  type Props = {
    tone?: "info" | "warning" | "success" | "error";
    size?: "xs" | "sm" | "md" | "lg";
    icon?: Component;
    iconClass?: string;
    class?: string;
    children?: Snippet;
  };

  let {
    tone = "info",
    size = "sm",
    icon,
    iconClass,
    class: klass,
    children,
  }: Props = $props();

  const sizeCls = $derived(
    {
      xs: "rounded-md px-3 py-2.5",
      sm: "rounded-lg p-3",
      md: "rounded-xl p-3 mb-4",
      lg: "rounded-2xl px-5 py-4 mb-6",
    }[size],
  );
  const containerTone = $derived(
    {
      info: "border-hairline-strong bg-surface-2",
      warning: "border-amber-500/40 bg-amber-500/10",
      success: "border-success/30 bg-success/10",
      error: "border-destructive/40 bg-destructive/10",
    }[tone],
  );
  const iconTone = $derived(
    {
      info: "text-muted-foreground",
      warning: "text-amber-600 dark:text-amber-400",
      success: "text-success",
      error: "text-destructive",
    }[tone],
  );
  const textTone = $derived(
    {
      info: "text-muted-foreground",
      warning: "text-amber-900 dark:text-amber-200",
      success: "text-foreground",
      error: "text-foreground",
    }[tone],
  );
  const DefaultIcon = $derived(
    icon ?? (tone === "warning" || tone === "error" ? AlertTriangle : Info),
  );
  const iconSize = $derived(size === "lg" ? "size-5" : "size-4");
  const gap = $derived(size === "sm" || size === "xs" ? "gap-2" : "gap-3");
</script>

<div class={cn("border flex items-start", containerTone, sizeCls, gap, klass)}>
  <DefaultIcon class={cn(iconSize, iconTone, "shrink-0 mt-0.5", iconClass)} />
  <div
    class={cn(
      "flex-1 leading-relaxed",
      size === "lg" ? "text-sm space-y-1" : "text-xs",
      textTone,
    )}
  >
    {@render children?.()}
  </div>
</div>
