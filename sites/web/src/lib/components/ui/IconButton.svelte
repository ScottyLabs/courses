<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLAnchorAttributes, HTMLButtonAttributes } from "svelte/elements";
  import type { IconComponent } from "./types";

  type Props = Omit<HTMLButtonAttributes, "class"> & {
    /** Accessible name; also shown as the native tooltip. Required: the button has no visible text. */
    label: string;
    icon?: IconComponent;
    /** sm 32px, md 40px, lg 44px. */
    size?: "sm" | "md" | "lg";
    variant?: "ghost" | "outline" | "solid";
    /** For toggle buttons: sets aria-pressed and the selected look. */
    pressed?: boolean;
    href?: string;
    /** Show `label` as a native tooltip on hover. */
    tooltip?: boolean;
    class?: string;
    children?: Snippet;
  };

  let {
    label,
    icon: Icon,
    size = "md",
    variant = "ghost",
    pressed,
    href,
    tooltip = true,
    type = "button",
    class: className = "",
    children,
    ...rest
  }: Props = $props();

  const sizes = {
    sm: "size-8 [&_svg]:size-4",
    md: "size-10 [&_svg]:size-5",
    lg: "size-11 [&_svg]:size-6",
  };
  const variants = {
    ghost: "border-transparent text-ink hover:bg-hover active:bg-active",
    outline: "border-line-strong text-ink hover:bg-hover active:bg-active",
    solid: "border-transparent bg-accent text-on-accent hover:bg-accent-hover",
  };
  const classes = $derived(
    [
      "inline-grid shrink-0 place-items-center rounded-full border transition-colors disabled:pointer-events-none disabled:opacity-50",
      sizes[size],
      variants[variant],
      pressed === true ? "bg-line text-strong" : "",
      className,
    ].join(" "),
  );
</script>

{#if href !== undefined}
  <a
    {href}
    aria-label={label}
    title={tooltip ? label : undefined}
    class={classes}
    {...rest as HTMLAnchorAttributes}
  >
    {#if Icon}<Icon aria-hidden="true" />{/if}
    {@render children?.()}
  </a>
{:else}
  <button
    {type}
    aria-label={label}
    aria-pressed={pressed}
    title={tooltip ? label : undefined}
    class={classes}
    {...rest}
  >
    {#if Icon}<Icon aria-hidden="true" />{/if}
    {@render children?.()}
  </button>
{/if}
