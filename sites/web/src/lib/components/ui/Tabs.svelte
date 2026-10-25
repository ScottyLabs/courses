<script lang="ts" module>
  import type { IconComponent } from "./types";

  export interface Tab {
    id: string;
    label: string;
    icon?: IconComponent;
    /** Small count after the label. */
    count?: number;
    disabled?: boolean;
  }
</script>

<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    tabs,
    value = $bindable(),
    label,
    panel,
    class: className = "",
  }: {
    tabs: readonly Tab[];
    /** Id of the selected tab; defaults to the first. */
    value?: string;
    /** Accessible name for the tab list. */
    label: string;
    /** Renders the selected tab's panel. */
    panel?: Snippet<[string]>;
    class?: string;
  } = $props();

  const baseId = $props.id();
  const selected = $derived(value ?? tabs.at(0)?.id ?? "");
  let list = $state<HTMLElement>();

  function select(index: number): void {
    const enabled = tabs.filter((tab) => tab.disabled !== true);
    const tab = enabled[((index % enabled.length) + enabled.length) % enabled.length];
    if (tab === undefined) return;
    value = tab.id;
    list?.querySelector<HTMLElement>(`#${CSS.escape(`${baseId}-tab-${tab.id}`)}`)?.focus();
  }

  function onkeydown(event: KeyboardEvent): void {
    const enabled = tabs.filter((tab) => tab.disabled !== true);
    const current = enabled.findIndex((tab) => tab.id === selected);
    if (event.key === "ArrowRight") select(current + 1);
    else if (event.key === "ArrowLeft") select(current - 1);
    else if (event.key === "Home") select(0);
    else if (event.key === "End") select(enabled.length - 1);
    else return;
    event.preventDefault();
  }
</script>

<div class={className}>
  <!-- The rule is on this wrapper so the scrolling strip can reach past it: the strip's padding leaves room for the
       focus ring, which goes around a tab's label (centered on the text, not the tall tab). -->
  <div class="border-b border-line">
    <div
      bind:this={list}
      role="tablist"
      aria-label={label}
      tabindex="-1"
      {onkeydown}
      class="scrollbar-thin -mx-2 -mb-px flex gap-6 overflow-x-auto px-2"
    >
      {#each tabs as tab (tab.id)}
        {@const isSelected = tab.id === selected}
        <button
          type="button"
          role="tab"
          id="{baseId}-tab-{tab.id}"
          aria-selected={isSelected}
          aria-controls="{baseId}-panel"
          tabindex={isSelected ? 0 : -1}
          disabled={tab.disabled}
          onclick={() => (value = tab.id)}
          class="group relative inline-flex shrink-0 items-center border-b-2 pt-1 pb-3 text-heading-6 transition-colors outline-none disabled:opacity-50 {isSelected
            ? 'border-accent text-accent-ink'
            : 'border-transparent text-muted hover:text-ink'}"
        >
          <span
            class="-mx-1 inline-flex items-center gap-2 rounded-tag px-1 group-focus-visible:animate-focus-ring group-focus-visible:outline-2 group-focus-visible:outline-offset-2 group-focus-visible:outline-accent"
          >
            {#if tab.icon}<tab.icon aria-hidden="true" class="size-5" />{/if}
            {tab.label}
            {#if tab.count !== undefined}
              <span class="tabular rounded-full bg-line px-1.5 text-label-md text-ink">{tab.count}</span>
            {/if}
          </span>
        </button>
      {/each}
    </div>
  </div>
  {#if panel}
    <div role="tabpanel" id="{baseId}-panel" aria-labelledby="{baseId}-tab-{selected}" class="pt-4">
      {@render panel(selected)}
    </div>
  {/if}
</div>
