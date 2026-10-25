<script lang="ts">
  import { ChevronLeft, ChevronRight } from "@lucide/svelte";
  import { tick, type Snippet } from "svelte";
  import { MediaQuery } from "svelte/reactivity";
  import Button from "../ui/Button.svelte";
  import IconButton from "../ui/IconButton.svelte";
  import Sheet from "../ui/Sheet.svelte";
  import { readLocal, writeLocal } from "../ui/storage";
  import type { IconComponent } from "../ui/types";
  import { RAIL_WIDTH, setRail } from "./rail-context";
  import { STABLE_GUTTER, stableGutter } from "./scrollbar-gutter";

  let {
    title = "Filter by",
    name = "filters",
    titleAction,
    rail,
    railFooter,
    showTitle = true,
    header,
    children,
    filterCount = 0,
    sheetLabel = "Filter",
    sheetButtonLabel,
    sheetIcon,
    backLabel = "Back",
    sheetDone,
    storageKey,
    autoCollapse = false,
    railCollapsed = $bindable(false),
    class: className = "",
  }: {
    /** Heading at the top of the rail (Figma "Filter by", "Schedule"). */
    title?: string;
    /** What the rail holds, for the Hide/Show buttons: "Hide filters". */
    name?: string;
    /** Small control after the title, such as the schedule page's add button. */
    titleAction?: Snippet;
    /** Rail content: RailGroup and RailSection blocks. Rendered once: in the column on desktop, in a sheet below it. */
    rail: Snippet;
    /** Pinned to the bottom of the rail (and of the phone sheet), such as a schedule's totals. */
    railFooter?: Snippet;
    /** False to keep the title for screen readers only, when the header already says where the student is. */
    showTitle?: boolean;
    /** White band at the top of the main column, usually a PageToolbar. */
    header?: Snippet;
    /** Main content on the page background. */
    children: Snippet;
    /** Active filters; shown on the phone Filter button and the collapsed rail. */
    filterCount?: number;
    /** Text on the phone button that opens the rail, and the sheet's title. */
    sheetLabel?: string;
    /** The phone button's whole text when it should differ from `sheetLabel` and the count, such as "Courses (3)". */
    sheetButtonLabel?: string;
    /** The phone button's icon in place of the filter icon. */
    sheetIcon?: IconComponent;
    /** Back button text in the phone sheet (Figma "< Search"). */
    backLabel?: string;
    /** A button at the foot of the phone sheet that closes it, such as "Show 85 courses". */
    sheetDone?: string;
    /** Remember "Hide" on this device under this key. */
    storageKey?: string;
    /**
     * Collapse the rail for now, such as while a course is open beside the list on a mid-size screen. Show still
     * opens it; nothing is remembered.
     */
    autoCollapse?: boolean;
    /** Bind to follow whether the rail column is collapsed (false below the desktop breakpoint). */
    railCollapsed?: boolean;
    class?: string;
  } = $props();

  const desktop = new MediaQuery("(min-width: 64rem)");
  const hiddenKey = $derived(storageKey === undefined ? undefined : `cmucourses:rail:${storageKey}`);
  function initiallyCollapsed(): boolean {
    return hiddenKey !== undefined && readLocal(hiddenKey) === "hidden";
  }

  /** Hidden with Hide, remembered on the device. */
  let collapsed = $state(initiallyCollapsed());
  /** Shown with Show while `autoCollapse` holds; forgotten when it ends. */
  let peek = $state(false);
  let sheetOpen = $state(false);

  let aside = $state<HTMLElement>();

  const shownCollapsed = $derived(collapsed || (autoCollapse && !peek));

  $effect.pre(() => {
    if (!autoCollapse) peek = false;
  });

  $effect.pre(() => {
    railCollapsed = desktop.current && shownCollapsed;
  });

  async function setCollapsed(next: boolean): Promise<void> {
    if (next && autoCollapse && peek && !collapsed) {
      // Hiding a rail shown over the automatic collapse just undoes that Show.
      peek = false;
    } else {
      collapsed = next;
      peek = !next && autoCollapse;
      if (hiddenKey !== undefined) writeLocal(hiddenKey, next ? "hidden" : null);
    }
    // The toggle button is replaced; keep keyboard focus on its counterpart.
    await tick();
    aside?.querySelector<HTMLElement>("[data-rail-toggle]")?.focus();
  }

  setRail({
    get asSheet() {
      return !desktop.current;
    },
    get count() {
      return filterCount;
    },
    get label() {
      return sheetLabel;
    },
    get buttonLabel() {
      return sheetButtonLabel;
    },
    get icon() {
      return sheetIcon;
    },
    open: () => {
      sheetOpen = true;
    },
    close: () => {
      sheetOpen = false;
    },
  });

  $effect(() => {
    if (desktop.current) sheetOpen = false;
  });
</script>

<div class="flex min-h-0 flex-1 {className}">
  {#if desktop.current}
    <aside
      bind:this={aside}
      aria-label={title}
      class="shrink-0 border-r border-line bg-surface {shownCollapsed ? 'w-16' : RAIL_WIDTH}"
    >
      <div
        {@attach stableGutter}
        class="scrollbar-thin sticky top-(--header-h) flex h-[calc(100dvh-var(--header-h))] flex-col overflow-y-auto overscroll-contain {STABLE_GUTTER}"
      >
        {#if shownCollapsed}
          <div class="flex flex-col items-center gap-2 pt-5">
            <IconButton
              label="Show {name}"
              icon={ChevronRight}
              aria-expanded="false"
              data-rail-toggle
              onclick={() => setCollapsed(false)}
            />
            {#if filterCount > 0}
              <span
                class="tabular grid size-6 place-items-center rounded-full bg-strong text-label-md text-surface"
                title="{filterCount} active"
              >
                {filterCount}<span class="sr-only">{` active ${name}`}</span>
              </span>
            {/if}
          </div>
        {:else}
          <div class="relative px-6 pt-[39px] min-[90rem]:px-[31px] {showTitle ? 'pb-6' : ''}">
            <div class={showTitle ? "flex items-center gap-2 pr-20" : "sr-only"}>
              <h2 class="text-heading-5 text-ink">{title}</h2>
              {@render titleAction?.()}
            </div>
            <Button
              variant="ghost"
              trailingIcon={ChevronLeft}
              aria-label="Hide {name}"
              aria-expanded="true"
              data-rail-toggle
              class="absolute top-[19px] right-2 min-[90rem]:right-[18px]"
              onclick={() => setCollapsed(true)}
            >
              Hide
            </Button>
          </div>
          <div>{@render rail()}</div>
          {#if railFooter}
            <div class="sticky bottom-0 mt-auto border-t border-line bg-surface px-6 py-4 min-[90rem]:px-[31px]">
              {@render railFooter()}
            </div>
          {/if}
        {/if}
      </div>
    </aside>
  {/if}

  <div class="flex min-w-0 flex-1 flex-col">
    {#if header}
      <div class="border-b border-line bg-surface px-4 py-5 sm:px-6 lg:px-8 lg:pt-[34px] lg:pb-[35px]">
        {@render header()}
      </div>
    {/if}
    <div class="flex-1 px-4 py-5 sm:px-6 lg:px-[30px] lg:py-8">
      {@render children()}
    </div>
  </div>
</div>

{#snippet sheetFooter()}
  {#if railFooter}<div class={sheetDone === undefined ? "" : "mb-3"}>{@render railFooter()}</div>{/if}
  {#if sheetDone !== undefined}
    <Button variant="primary" full onclick={() => (sheetOpen = false)}>{sheetDone}</Button>
  {/if}
{/snippet}

{#if !desktop.current}
  <Sheet
    bind:open={sheetOpen}
    title={sheetLabel}
    {backLabel}
    side="right"
    footer={sheetDone === undefined && railFooter === undefined ? undefined : sheetFooter}
  >
    <div class="pt-2">{@render rail()}</div>
  </Sheet>
{/if}
