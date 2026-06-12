<script lang="ts" module>
  import { tv, type VariantProps } from "tailwind-variants";

  export const badgeVariants = tv({
    base: "inline-flex items-center gap-1 rounded-md border px-2 py-0.5 text-xs font-medium",
    variants: {
      variant: {
        default: "border-transparent bg-primary/15 text-primary",
        secondary: "border-transparent bg-secondary text-secondary-foreground",
        outline: "text-foreground",
        success: "border-transparent bg-success/15 text-success",
        warning: "border-transparent bg-amber-500/15 text-amber-700 dark:text-amber-300",
        destructive: "border-transparent bg-destructive/15 text-destructive",
      },
    },
    defaultVariants: { variant: "default" },
  });

  export type BadgeVariant = VariantProps<typeof badgeVariants>["variant"];
</script>

<script lang="ts">
  import type { Snippet } from "svelte";
  import { cn } from "$lib/utils";

  let {
    variant = "default",
    class: klass,
    children,
  }: { variant?: BadgeVariant; class?: string; children?: Snippet } = $props();
</script>

<span class={cn(badgeVariants({ variant }), klass)}>
  {@render children?.()}
</span>
