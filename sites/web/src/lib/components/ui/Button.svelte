<script lang="ts" module>
  export type ButtonVariant = "primary" | "secondary" | "outline" | "ghost" | "danger";
  export type ButtonSize = "sm" | "md" | "lg";
</script>

<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLAnchorAttributes, HTMLButtonAttributes } from "svelte/elements";
  import Spinner from "./Spinner.svelte";
  import type { IconComponent } from "./types";

  type Props = Omit<HTMLButtonAttributes, "class"> & {
    /** primary: filled accent; secondary: soft fill; outline: bordered pill (the Figma default); ghost: text only; danger: destructive. */
    variant?: ButtonVariant;
    size?: ButtonSize;
    /** Render a link styled as a button. */
    href?: string;
    target?: HTMLAnchorAttributes["target"];
    rel?: string;
    download?: HTMLAnchorAttributes["download"];
    icon?: IconComponent;
    trailingIcon?: IconComponent;
    /** Shows a spinner in place of the icon and marks the button busy. */
    loading?: boolean;
    /** Stretch to the container's width. */
    full?: boolean;
    class?: string;
    children?: Snippet;
  };

  let {
    variant = "outline",
    size = "md",
    href,
    target,
    rel,
    download,
    icon: Icon,
    trailingIcon: TrailingIcon,
    loading = false,
    full = false,
    disabled = false,
    type = "button",
    class: className = "",
    children,
    ...rest
  }: Props = $props();

  const variants: Record<ButtonVariant, string> = {
    primary: "border-transparent bg-accent text-on-accent hover:bg-accent-hover",
    secondary: "border-transparent bg-paper text-ink hover:bg-line active:bg-pressed",
    outline: "border-line-strong bg-transparent text-ink hover:bg-hover active:bg-active",
    ghost: "border-transparent bg-transparent text-ink hover:bg-hover active:bg-active",
    danger: "border-danger/35 bg-danger-bg text-danger hover:border-danger",
  };
  const sizes: Record<ButtonSize, string> = {
    sm: "h-8 gap-1.5 px-3 text-label-md [&_svg]:size-4",
    md: "h-10 gap-2 px-4 text-label-lg [&_svg]:size-5",
    lg: "h-12 gap-2 px-6 text-label-lg text-[16px] [&_svg]:size-5",
  };

  const classes = $derived(
    [
      "inline-flex shrink-0 items-center justify-center whitespace-nowrap rounded-full border transition-colors duration-150 select-none",
      "disabled:pointer-events-none disabled:opacity-50 aria-disabled:pointer-events-none aria-disabled:opacity-50",
      variants[variant],
      sizes[size],
      full ? "w-full" : "",
      className,
    ].join(" "),
  );
  const external = $derived(href !== undefined && /^https?:/u.test(href));
</script>

{#snippet content()}
  {#if loading}
    <Spinner size="sm" decorative />
  {:else if Icon}
    <Icon aria-hidden="true" />
  {/if}
  {@render children?.()}
  {#if TrailingIcon}
    <TrailingIcon aria-hidden="true" />
  {/if}
{/snippet}

{#if href !== undefined}
  <a
    {href}
    target={target ?? (external ? "_blank" : undefined)}
    rel={rel ?? (external ? "noopener noreferrer" : undefined)}
    {download}
    class={classes}
    aria-disabled={disabled || undefined}
    aria-busy={loading || undefined}
    {...rest as HTMLAnchorAttributes}
  >
    {@render content()}
  </a>
{:else}
  <button
    {type}
    class={classes}
    disabled={disabled || loading}
    aria-busy={loading || undefined}
    {...rest}
  >
    {@render content()}
  </button>
{/if}
