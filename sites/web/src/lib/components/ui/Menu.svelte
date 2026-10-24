<script lang="ts" module>
  import type { IconComponent } from "./types";

  export type MenuEntry =
    | {
        label: string;
        icon?: IconComponent;
        /** Runs when the item is chosen; the menu then closes. */
        onselect?: () => void;
        /** Makes the item a link. */
        href?: string;
        danger?: boolean;
        disabled?: boolean;
      }
    | "separator";
</script>

<script lang="ts">
  import type { Snippet } from "svelte";
  import type { Placement } from "./floating";
  import Popover, { type PopoverTriggerProps } from "./Popover.svelte";

  let {
    items,
    label,
    placement = "bottom-end",
    open = $bindable(false),
    trigger,
    class: className = "",
  }: {
    items: readonly MenuEntry[];
    /** Accessible name for the menu. */
    label: string;
    placement?: Placement;
    open?: boolean;
    trigger: Snippet<[PopoverTriggerProps]>;
    class?: string;
  } = $props();

  let list = $state<HTMLElement>();
  let startAtEnd = false;

  function menuItems(): HTMLElement[] {
    return [
      ...(list?.parentElement?.querySelectorAll<HTMLElement>(
        '[role="menuitem"]:not([aria-disabled="true"])',
      ) ?? []),
    ];
  }

  function focusItem(index: number): void {
    const all = menuItems();
    if (all.length === 0) return;
    all[((index % all.length) + all.length) % all.length]?.focus();
  }

  function onkeydown(event: KeyboardEvent): void {
    const all = menuItems();
    const current = all.indexOf(document.activeElement as HTMLElement);
    if (event.key === "ArrowDown") focusItem(current + 1);
    else if (event.key === "ArrowUp") focusItem(current < 0 ? -1 : current - 1);
    else if (event.key === "Home") focusItem(0);
    else if (event.key === "End") focusItem(-1);
    else if (event.key === "Tab") open = false;
    else return;
    if (event.key !== "Tab") event.preventDefault();
  }

  function ontriggerkeydown(event: KeyboardEvent): void {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      startAtEnd = event.key === "ArrowUp";
      open = true;
    }
  }

  $effect(() => {
    if (!open || list === undefined) return;
    const atEnd = startAtEnd;
    startAtEnd = false;
    requestAnimationFrame(() => focusItem(atEnd ? -1 : 0));
  });
</script>

<Popover
  bind:open
  {placement}
  {label}
  role="menu"
  focusOnOpen={false}
  {trigger}
  {ontriggerkeydown}
  class="min-w-48 py-1.5 {className}"
>
  {#snippet children({ close })}
    <span bind:this={list} hidden></span>
    {#each items as item, index (index)}
      {#if item === "separator"}
        <div role="separator" class="my-1.5 h-px bg-line"></div>
      {:else}
        {@const Icon = item.icon}
        {@const classes = `flex w-full items-center gap-3 px-3.5 py-2 text-left text-body-2 outline-none hover:bg-paper focus:bg-paper aria-disabled:pointer-events-none aria-disabled:text-faint ${item.danger === true ? "text-danger" : "text-ink"}`}
        {#if item.href !== undefined}
          <a
            role="menuitem"
            tabindex="-1"
            href={item.href}
            aria-disabled={item.disabled}
            class={classes}
            {onkeydown}
            onclick={() => close(false)}
          >
            {#if Icon}<Icon aria-hidden="true" class="size-4 shrink-0" />{/if}{item.label}
          </a>
        {:else}
          <button
            type="button"
            role="menuitem"
            tabindex="-1"
            aria-disabled={item.disabled}
            class={classes}
            {onkeydown}
            onclick={() => {
              close(true);
              item.onselect?.();
            }}
          >
            {#if Icon}<Icon aria-hidden="true" class="size-4 shrink-0" />{/if}{item.label}
          </button>
        {/if}
      {/if}
    {/each}
  {/snippet}
</Popover>
