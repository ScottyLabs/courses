<script lang="ts">
  import { Check, ChevronDown } from "@lucide/svelte";
  import { untrack } from "svelte";
  import { follow, hidePanel, showPanel } from "./floating";
  import { restoreFocus } from "./focus";
  import { matchesQuery, step } from "./listbox";
  import { listRow, panelSurface } from "./styles";
  import type { Option } from "./types";

  let {
    options,
    value = $bindable([]),
    label,
    hideLabel = true,
    placeholder = "Select",
    searchPlaceholder = "Search",
    emptyText = "No matches",
    disabled = false,
    limit = 200,
    onchange,
    class: className = "",
  }: {
    options: readonly Option[];
    /** Values of the chosen options. */
    value?: string[];
    label: string;
    hideLabel?: boolean;
    /** Trigger text while nothing is chosen. */
    placeholder?: string;
    searchPlaceholder?: string;
    emptyText?: string;
    disabled?: boolean;
    limit?: number;
    onchange?: (value: string[]) => void;
    class?: string;
  } = $props();

  const baseId = $props.id();
  const labelId = `${baseId}-label`;
  const triggerId = `${baseId}-trigger`;
  const listId = `${baseId}-list`;
  let trigger = $state<HTMLButtonElement>();
  let panel = $state<HTMLElement>();
  let search = $state<HTMLInputElement>();
  let open = $state(false);
  let query = $state("");
  let active = $state(-1);

  const chosen = $derived(new Set(value));
  const filtered = $derived(options.filter((option) => matchesQuery(option, query)).slice(0, limit));
  const summary = $derived.by(() => {
    if (value.length === 0) return placeholder;
    if (value.length === 1) return options.find((option) => option.value === value[0])?.label ?? "1 selected";
    return `${value.length} selected`;
  });

  function optionId(index: number): string {
    return `${baseId}-option-${index}`;
  }

  function toggle(option: Option): void {
    if (option.disabled === true) return;
    value = chosen.has(option.value)
      ? value.filter((item) => item !== option.value)
      : [...value, option.value];
    onchange?.(value);
  }

  function close(focusTrigger: boolean): void {
    open = false;
    query = "";
    active = -1;
    if (focusTrigger) restoreFocus(trigger ?? null);
  }

  function onsearchkeydown(event: KeyboardEvent): void {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      active = step(filtered, active, event.key === "ArrowDown" ? 1 : -1);
    } else if (event.key === "Enter") {
      event.preventDefault();
      const option = filtered[active];
      if (option !== undefined) toggle(option);
    } else if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      close(true);
    }
  }

  $effect(() => {
    if (!open || trigger === undefined || panel === undefined) return;
    const triggerElement = trigger;
    const panelElement = panel;
    showPanel(panelElement);
    const stop = untrack(() => follow(triggerElement, panelElement, { matchWidth: true, offset: 4 }));
    requestAnimationFrame(() => search?.focus());
    const onPointerDown = (event: PointerEvent): void => {
      const target = event.target as Node;
      if (!triggerElement.contains(target) && !panelElement.contains(target)) close(false);
    };
    document.addEventListener("pointerdown", onPointerDown, true);
    return () => {
      stop();
      hidePanel(panelElement);
      document.removeEventListener("pointerdown", onPointerDown, true);
    };
  });

  $effect(() => {
    if (open && active >= 0) {
      document.getElementById(optionId(active))?.scrollIntoView({ block: "nearest" });
    }
  });
</script>

<div class={className}>
  <span id={labelId} class={hideLabel ? "sr-only" : "mb-1.5 block text-label-lg text-ink"}>{label}</span>
  <button
    bind:this={trigger}
    id={triggerId}
    type="button"
    aria-haspopup="listbox"
    aria-expanded={open}
    aria-labelledby="{labelId} {triggerId}"
    {disabled}
    onclick={() => (open ? close(false) : (open = true))}
    onkeydown={(event) => {
      if (event.key === "ArrowDown" && !open) {
        event.preventDefault();
        open = true;
      }
    }}
    class="focus-field flex h-10 w-full items-center justify-between gap-2 rounded-field border bg-surface px-4 text-left text-body-2 disabled:border-line disabled:text-faint {open
      ? 'border-accent shadow-[inset_0_0_0_1px_var(--color-accent)]'
      : 'border-line-strong hover:border-muted'}"
  >
    <span class="truncate {value.length === 0 ? 'text-muted' : 'text-ink'}">{summary}</span>
    <ChevronDown
      aria-hidden="true"
      class="size-5 shrink-0 transition-transform {open ? 'rotate-180' : ''} {disabled
        ? 'text-faint'
        : 'text-ink'}"
    />
  </button>
  <div
    bind:this={panel}
    popover="manual"
    data-floating
    class="{panelSurface} flex flex-col overflow-hidden"
    onfocusout={(event) => {
      const next = event.relatedTarget as Node | null;
      if (next !== null && !panel?.contains(next) && next !== trigger) close(false);
    }}
  >
    <div class="border-b border-line p-2">
      <input
        bind:this={search}
        bind:value={query}
        type="text"
        role="combobox"
        aria-expanded="true"
        aria-controls={listId}
        aria-autocomplete="list"
        aria-activedescendant={active >= 0 ? optionId(active) : undefined}
        aria-label="Search {label}"
        placeholder={searchPlaceholder}
        autocomplete="off"
        spellcheck="false"
        oninput={() => (active = step(filtered, -1, 1))}
        onkeydown={onsearchkeydown}
        class="focus-field h-9 w-full rounded-[8px] border border-line-strong bg-surface px-3 text-body-2 text-ink placeholder:text-muted"
      />
    </div>
    <ul
      id={listId}
      role="listbox"
      aria-multiselectable="true"
      aria-label={label}
      class="scrollbar-thin max-h-[min(16rem,calc(var(--floating-available)-6.5rem))] overflow-y-auto py-1.5"
    >
      {#each filtered as option, index (option.value)}
        {@const isChosen = chosen.has(option.value)}
        <!-- Keyboard selection happens in the search input (aria-activedescendant). -->
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <li
          id={optionId(index)}
          role="option"
          aria-selected={isChosen}
          aria-disabled={option.disabled}
          data-active={index === active}
          class={listRow}
          onpointerdown={(event) => event.preventDefault()}
          onclick={() => toggle(option)}
          onpointermove={() => (active = index)}
        >
          <span
            aria-hidden="true"
            class="grid size-4 shrink-0 place-items-center rounded-[4px] border {isChosen
              ? 'border-accent bg-accent text-on-accent'
              : 'border-ink'}"
          >
            {#if isChosen}<Check class="size-3" strokeWidth={3} />{/if}
          </span>
          <span class="min-w-0 flex-1">
            <span class="block truncate">{option.label}</span>
            {#if option.description}
              <span class="block truncate text-label-md font-normal text-muted">{option.description}</span>
            {/if}
          </span>
        </li>
      {:else}
        <li role="presentation" class="px-3.5 py-2 text-body-2 text-muted">{emptyText}</li>
      {/each}
    </ul>
    <div class="flex items-center justify-between border-t border-line px-3.5 py-2">
      <span class="text-label-md text-muted">{value.length} selected</span>
      <button
        type="button"
        class="rounded-full px-2 py-1 text-label-md text-accent-ink hover:bg-hover disabled:text-faint"
        disabled={value.length === 0}
        onclick={() => {
          value = [];
          onchange?.(value);
        }}
      >
        Clear
      </button>
    </div>
  </div>
</div>
