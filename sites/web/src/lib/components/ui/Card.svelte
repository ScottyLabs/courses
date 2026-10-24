<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";

  type Props = Omit<HTMLAttributes<HTMLElement>, "class"> & {
    as?: "div" | "article" | "section" | "li";
    /** Makes the whole card a link. Prefer a link on the title when the card has other controls. */
    href?: string;
    /** none; sm 16px; md 20px (saved course card); lg 30/36px (course card). */
    padding?: "none" | "sm" | "md" | "lg";
    /** Hover feedback for cards that open something. */
    interactive?: boolean;
    class?: string;
    children?: Snippet;
  };

  let {
    as = "div",
    href,
    padding = "md",
    interactive = false,
    class: className = "",
    children,
    ...rest
  }: Props = $props();

  const paddings = {
    none: "",
    sm: "p-4",
    md: "p-5",
    lg: "px-4 py-5 sm:px-9 sm:py-[30px]",
  };
  const classes = $derived(
    [
      "block rounded-card border border-pressed bg-surface text-ink",
      paddings[padding],
      interactive || href !== undefined ? "transition-colors hover:border-line-strong" : "",
      className,
    ].join(" "),
  );
</script>

{#if href !== undefined}
  <a {href} class={classes} {...rest}>{@render children?.()}</a>
{:else}
  <svelte:element this={as} class={classes} {...rest}>{@render children?.()}</svelte:element>
{/if}
