<script lang="ts">
  // One action in an OptionsMenu: a button, or a link with `href`.
  import type { Snippet } from "svelte";
  import type { IconComponent } from "$lib/components/ui/types";

  let {
    icon: Icon,
    href,
    danger = false,
    disabled = false,
    onclick,
    children,
  }: {
    icon?: IconComponent;
    href?: string;
    danger?: boolean;
    disabled?: boolean;
    onclick?: () => void;
    children: Snippet;
  } = $props();

  const classes = $derived(
    `flex min-h-10 w-full items-center gap-3 px-3.5 py-1.5 text-left text-body-2 outline-none hover:bg-paper focus-visible:bg-paper disabled:pointer-events-none disabled:text-faint md:min-h-9 [&_svg]:size-4 [&_svg]:shrink-0 ${danger ? "text-danger" : "text-ink"}`,
  );
</script>

{#if href !== undefined}
  <a {href} class={classes} onclick={() => onclick?.()}>
    {#if Icon}<Icon aria-hidden="true" class={danger ? "" : "text-muted"} />{/if}
    {@render children()}
  </a>
{:else}
  <button type="button" class={classes} {disabled} onclick={() => onclick?.()}>
    {#if Icon}<Icon aria-hidden="true" class={danger ? "" : "text-muted"} />{/if}
    {@render children()}
  </button>
{/if}
