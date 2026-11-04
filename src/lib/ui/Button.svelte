<script lang="ts" module>
  import { tv, type VariantProps } from "tailwind-variants";

  export const buttonVariants = tv({
    // Fluent buttons: 4px radius (rounded-sm), 1px stroke, no press-scale —
    // pressed state flattens via a darker fill. Focus shows a 2px accent ring
    // offset from the control (Fluent focus rectangle).
    base: "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-sm text-sm font-medium transition-colors duration-150 disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-4 shrink-0 [&_svg]:shrink-0 outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background cursor-pointer",
    variants: {
      variant: {
        // Fluent AccentButton disabled state: neutral fill + dimmed text, not a faded accent.
        default:
          "bg-primary text-primary-foreground shadow-sm hover:bg-primary/90 active:bg-primary/80 disabled:opacity-100 disabled:bg-foreground/15 disabled:text-foreground/45 disabled:shadow-none",
        destructive:
          "bg-destructive text-white shadow-sm hover:bg-destructive/90 active:bg-destructive/80 focus-visible:ring-destructive",
        outline:
          "border border-hairline-strong bg-card text-foreground shadow-sm hover:bg-surface-3 active:bg-surface-1",
        secondary:
          "border border-hairline bg-surface-2 text-foreground hover:bg-surface-3 active:bg-surface-1",
        ghost: "text-foreground hover:bg-surface-3 active:bg-surface-1",
        link: "text-primary underline-offset-4 hover:underline",
      },
      size: {
        default: "h-8 px-4 has-[>svg]:px-3",
        sm: "h-7 gap-1.5 px-3 has-[>svg]:px-2.5",
        lg: "h-9 px-6 has-[>svg]:px-4",
        icon: "size-8",
      },
    },
    defaultVariants: { variant: "default", size: "default" },
  });

  export type ButtonVariant = VariantProps<typeof buttonVariants>["variant"];
  export type ButtonSize = VariantProps<typeof buttonVariants>["size"];
</script>

<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes, HTMLAnchorAttributes } from "svelte/elements";
  import { cn } from "$lib/utils";

  type Props = {
    variant?: ButtonVariant;
    size?: ButtonSize;
    href?: string;
    class?: string;
    children?: Snippet;
  } & (HTMLButtonAttributes | HTMLAnchorAttributes);

  let {
    variant = "default",
    size = "default",
    href,
    class: klass,
    children,
    ...rest
  }: Props = $props();
</script>

{#if href}
  <a {href} class={cn(buttonVariants({ variant, size }), klass)} {...rest as HTMLAnchorAttributes}>
    {@render children?.()}
  </a>
{:else}
  <button class={cn(buttonVariants({ variant, size }), klass)} {...rest as HTMLButtonAttributes}>
    {@render children?.()}
  </button>
{/if}
