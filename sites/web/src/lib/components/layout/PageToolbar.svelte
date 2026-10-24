<script lang="ts">
  import type { Snippet } from "svelte";
  import Chip from "../ui/Chip.svelte";
  import FilterButton from "./FilterButton.svelte";
  import { getRail } from "./rail-context";

  let {
    title,
    hideTitle = false,
    titleAction,
    intro,
    search,
    count,
    countLabel = "results",
    chips,
    onreset,
    actions,
    actionsLayout = "auto",
    children,
    class: className = "",
  }: {
    /** Page heading (h1). */
    title?: string;
    /** "desktop" keeps it for screen readers only on wide screens (the courses page shows "Search" on phones only). */
    hideTitle?: boolean | "desktop";
    /** Controls after the title, such as the schedule rename button or Friends' "Add Friends". */
    titleAction?: Snippet;
    /** Content between the heading and the search, such as instructions and action buttons. */
    intro?: Snippet;
    /** The search field, usually a SearchInput. */
    search?: Snippet;
    /** Result count; numbers are formatted ("7,927 results"). */
    count?: number | string;
    countLabel?: string;
    /** Removable filter chips. */
    chips?: Snippet;
    /** Shows a Reset pill after the chips at phone widths, where the rail's Reset is in the sheet. */
    onreset?: () => void;
    /** Right-aligned controls: sort select, view toggle. */
    actions?: Snippet;
    /**
     * Where `actions` go on small screens. auto: beside the chips, wrapping under them when they don't fit. inline:
     * in the Filter button's row while the rail is a sheet (for a few icon buttons). row: a full row of their own on
     * phones, so they never squeeze the result count.
     */
    actionsLayout?: "auto" | "inline" | "row";
    /** Anything else under the toolbar, such as the profile filters row or a workload summary. */
    children?: Snippet;
    class?: string;
  } = $props();

  const rail = getRail();
  const countText = $derived(
    typeof count === "number" ? `${count.toLocaleString("en-US")} ${countLabel}` : count,
  );
  const hasMeta = $derived(
    count !== undefined || chips !== undefined || actions !== undefined || rail?.asSheet === true,
  );
  const actionsInline = $derived(
    actionsLayout === "inline" && actions !== undefined && rail?.asSheet === true,
  );
  const titleClass = $derived(
    hideTitle === true ? "sr-only" : hideTitle === "desktop" ? "lg:sr-only" : "",
  );
</script>

<div class="flex flex-col {className}">
  {#if title}
    <div class="mb-2 flex items-center gap-3 {titleClass} lg:mb-4">
      <h1 class="text-heading-3 text-strong">{title}</h1>
      {@render titleAction?.()}
    </div>
  {/if}
  {@render intro?.()}
  {@render search?.()}
  {#if hasMeta}
    <div class="mt-3 flex flex-wrap items-start justify-between gap-x-4 gap-y-3 lg:mt-3.5">
      <div class="flex min-w-0 flex-1 flex-col gap-[11px]">
        {#if rail?.asSheet || count !== undefined}
          <div class="flex min-h-8 items-center gap-3 lg:min-h-5">
            <FilterButton />
            {#if countText !== undefined}
              <p class="ml-auto text-label-md text-muted lg:ml-0 lg:text-label-lg" aria-live="polite">
                {countText}
              </p>
            {/if}
            {#if actionsInline}
              <div class="ml-auto flex min-w-0 flex-wrap items-center justify-end gap-2">{@render actions?.()}</div>
            {/if}
          </div>
        {/if}
        {#if chips || (onreset && rail?.asSheet)}
          <div class="flex flex-wrap items-center gap-2 lg:gap-[7px]">
            {@render chips?.()}
            {#if onreset && rail?.asSheet}
              <Chip tone="accent" shape="pill" class="px-4" onclick={onreset}>Reset</Chip>
            {/if}
          </div>
        {/if}
      </div>
      {#if actions && !actionsInline}
        <div
          class="flex max-w-full shrink-0 flex-wrap items-center gap-3 {actionsLayout === 'row' ? 'max-sm:w-full' : ''}"
        >
          {@render actions()}
        </div>
      {/if}
    </div>
  {/if}
  {@render children?.()}
</div>
