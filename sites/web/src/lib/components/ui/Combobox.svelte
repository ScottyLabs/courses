<script lang="ts">
  import { Check, ChevronsUpDown, X } from "@lucide/svelte";
  import { untrack } from "svelte";
  import { follow, hidePanel, showPanel } from "./floating";
  import { matchesQuery, step } from "./listbox";
  import { listRow, panelSurface } from "./styles";
  import type { Option } from "./types";

  let {
    options,
    value = $bindable(null),
    label,
    hideLabel = true,
    placeholder = "Select",
    emptyText = "No matches",
    disabled = false,
    clearable = true,
    limit = 200,
    onchange,
    class: className = "",
  }: {
    options: readonly Option[];
    /** The chosen option's value, or null. */
    value?: string | null;
    label: string;
    hideLabel?: boolean;
    placeholder?: string;
    /** Shown when nothing matches the typed text. */
    emptyText?: string;
    disabled?: boolean;
    /** Show a clear button while something is chosen. */
    clearable?: boolean;
    /** Most options rendered at once; typing narrows the rest. */
    limit?: number;
    onchange?: (value: string | null) => void;
    class?: string;
  } = $props();

  const baseId = $props.id();
  const inputId = `${baseId}-input`;
  const listId = `${baseId}-list`;
  let input = $state<HTMLInputElement>();
  let field = $state<HTMLElement>();
  let panel = $state<HTMLElement>();
  let open = $state(false);
  let query = $state("");
  let active = $state(-1);

  const selected = $derived(options.find((option) => option.value === value));
  const filtered = $derived(options.filter((option) => matchesQuery(option, query)).slice(0, limit));

  function optionId(index: number): string {
    return `${baseId}-option-${index}`;
  }

  function show(): void {
    if (disabled || open) return;
    query = "";
    open = true;
    const current = options.findIndex((option) => option.value === value);
    active = current >= 0 && current < limit ? current : step(options.slice(0, limit), -1, 1);
  }

  function hide(): void {
    open = false;
    query = "";
    active = -1;
  }

  function choose(option: Option): void {
    if (option.disabled === true) return;
    value = option.value;
    onchange?.(option.value);
    hide();
  }

  function clear(): void {
    value = null;
    onchange?.(null);
    input?.focus();
  }

  function oninput(event: Event & { currentTarget: HTMLInputElement }): void {
    query = event.currentTarget.value;
    open = true;
    active = step(
      options.filter((option) => matchesQuery(option, query)).slice(0, limit),
      -1,
      1,
    );
  }

  function onkeydown(event: KeyboardEvent): void {
    switch (event.key) {
      case "ArrowDown":
      case "ArrowUp": {
        event.preventDefault();
        if (open) active = step(filtered, active, event.key === "ArrowDown" ? 1 : -1);
        else show();
        break;
      }
      case "Home":
      case "End": {
        if (!open) return;
        event.preventDefault();
        active = event.key === "Home" ? step(filtered, -1, 1) : step(filtered, filtered.length, -1);
        break;
      }
      case "Enter": {
        const option = filtered[active];
        if (open && option !== undefined) {
          event.preventDefault();
          choose(option);
        }
        break;
      }
      case "Escape": {
        if (open) {
          event.preventDefault();
          event.stopPropagation();
          hide();
        }
        break;
      }
      default:
        break;
    }
  }

  $effect(() => {
    if (!open || field === undefined || panel === undefined) return;
    const panelElement = panel;
    showPanel(panelElement);
    const stop = untrack(() => follow(field as HTMLElement, panelElement, { matchWidth: true, offset: 4 }));
    return () => {
      stop();
      hidePanel(panelElement);
    };
  });

  $effect(() => {
    if (open && active >= 0) {
      document.getElementById(optionId(active))?.scrollIntoView({ block: "nearest" });
    }
  });
</script>

<div class={className}>
  <label for={inputId} class={hideLabel ? "sr-only" : "mb-1.5 block text-label-lg text-ink"}>{label}</label>
  <div bind:this={field} class="relative">
    <input
      bind:this={input}
      id={inputId}
      type="text"
      role="combobox"
      aria-expanded={open}
      aria-controls={listId}
      aria-autocomplete="list"
      aria-activedescendant={open && active >= 0 ? optionId(active) : undefined}
      autocomplete="off"
      spellcheck="false"
      value={open ? query : (selected?.label ?? "")}
      placeholder={open && selected ? selected.label : placeholder}
      {disabled}
      {oninput}
      {onkeydown}
      onclick={show}
      onblur={hide}
      class="h-10 w-full rounded-field border border-line-strong bg-surface pl-4 text-body-2 text-ink transition-colors outline-none placeholder:text-muted hover:border-muted focus:border-accent disabled:border-line disabled:text-faint disabled:placeholder:text-faint {clearable &&
      selected
        ? 'pr-[4.5rem]'
        : 'pr-11'}"
    />
    {#if clearable && selected && !disabled}
      <button
        type="button"
        aria-label="Clear {label}"
        title="Clear"
        onclick={clear}
        class="absolute top-1/2 right-10 grid size-7 -translate-y-1/2 place-items-center rounded-full text-muted hover:bg-hover hover:text-ink"
      >
        <X aria-hidden="true" class="size-4" />
      </button>
    {/if}
    <ChevronsUpDown
      aria-hidden="true"
      class="pointer-events-none absolute top-1/2 right-4 size-5 -translate-y-1/2 {disabled
        ? 'text-faint'
        : 'text-ink'}"
    />
  </div>
  <div
    bind:this={panel}
    popover="manual"
    data-floating
    class="{panelSurface} overflow-hidden"
  >
    <!-- Pointer presses in the list must not blur the input, which would close the list. -->
    <ul
      id={listId}
      role="listbox"
      aria-label={label}
      onpointerdown={(event) => event.preventDefault()}
      class="scrollbar-thin max-h-[min(18rem,var(--floating-available))] overflow-y-auto py-1.5"
    >
      {#each filtered as option, index (option.value)}
        <!-- Keyboard selection happens in the combobox input (aria-activedescendant). -->
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <li
          id={optionId(index)}
          role="option"
          aria-selected={option.value === value}
          aria-disabled={option.disabled}
          data-active={index === active}
          class={listRow}
          onclick={() => choose(option)}
          onpointermove={() => (active = index)}
        >
          <span class="min-w-0 flex-1">
            <span class="block truncate">{option.label}</span>
            {#if option.description}
              <span class="block truncate text-label-md font-normal text-muted">{option.description}</span>
            {/if}
          </span>
          {#if option.value === value}
            <Check aria-hidden="true" class="size-4 shrink-0 text-accent-ink" />
          {/if}
        </li>
      {:else}
        <li role="presentation" class="px-3.5 py-2 text-body-2 text-muted">{emptyText}</li>
      {/each}
    </ul>
  </div>
</div>
