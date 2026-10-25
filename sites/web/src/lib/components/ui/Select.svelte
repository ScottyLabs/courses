<script lang="ts" module>
  import type { Attachment } from "svelte/attachments";

  /** Spread these onto the button a `trigger` snippet renders. */
  export interface SelectTriggerProps {
    id: string;
    type: "button";
    role: "combobox";
    "aria-haspopup": "listbox";
    "aria-expanded": boolean;
    "aria-controls": string;
    "aria-labelledby": string;
    "aria-activedescendant": string | undefined;
    disabled: boolean;
    onclick: () => void;
    onkeydown: (event: KeyboardEvent) => void;
    onkeyup: (event: KeyboardEvent) => void;
    onblur: (event: FocusEvent) => void;
    [key: symbol]: Attachment<HTMLElement>;
  }

  /** What `onchange` receives: the part of a native select's change event that handlers read, so handlers written
   * for one (`event.currentTarget.value`) keep working. */
  export interface SelectChangeEvent {
    readonly currentTarget: { readonly value: string };
  }
</script>

<script lang="ts">
  // A select-only combobox (the WAI-ARIA pattern): a button showing the chosen option that opens a listbox in a
  // floating panel. Focus stays on the button while the list is open; aria-activedescendant points at the active
  // option. Built instead of a native <select> so the closed field and the open list match Combobox and MultiSelect
  // in every browser (decision 30).
  import { Check, ChevronDown } from "@lucide/svelte";
  import { untrack, type Snippet } from "svelte";
  import { createAttachmentKey } from "svelte/attachments";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { follow, hidePanel, showPanel, type Placement } from "./floating";
  import { move, typeahead } from "./listbox";
  import { listRow, panelSurface } from "./styles";
  import type { Option } from "./types";

  type Props = Omit<HTMLButtonAttributes, "class" | "value" | "type" | "name" | "onchange" | "children"> & {
    value?: string;
    options?: readonly Option[];
    /** Shown while `value` is "". */
    placeholder?: string;
    label: string;
    hideLabel?: boolean;
    /** field: the 40px rail field; compact: the 36px toolbar dropdown (40px on phones). */
    variant?: "field" | "compact";
    /** Submits the value with a form under this name. */
    name?: string;
    /** Where the list opens; it flips above the field when there is no room below. */
    placement?: Placement;
    onchange?: (event: SelectChangeEvent) => void;
    /** Renders the button instead of the field, for a select shown as an icon or inline text: spread the props onto
     * a button. The second argument is the text the field would show (the chosen option's label, or the
     * placeholder). Other attributes, such as aria-describedby, go on the field only. */
    trigger?: Snippet<[SelectTriggerProps, string]>;
    class?: string;
  };

  let {
    value = $bindable(""),
    options = [],
    placeholder,
    label,
    hideLabel = true,
    variant = "field",
    disabled = false,
    name,
    placement = "bottom-start",
    id: providedId,
    onchange,
    trigger,
    class: className = "",
    ...rest
  }: Props = $props();

  const baseId = $props.id();
  const id = $derived(providedId ?? `${baseId}-trigger`);
  const labelId = `${baseId}-label`;
  const listId = `${baseId}-list`;
  let button = $state<HTMLElement>();
  let panel = $state<HTMLElement>();
  let list = $state<HTMLElement>();
  let open = $state(false);
  let active = $state(-1);
  let search = "";
  let searchTimer: ReturnType<typeof setTimeout> | undefined;

  const selected = $derived(options.find((option) => option.value === value));
  const shown = $derived(
    value === "" && placeholder !== undefined ? placeholder : (selected?.label ?? placeholder ?? ""),
  );

  function optionId(index: number): string {
    return `${baseId}-option-${index}`;
  }

  function show(start: number | null = null): void {
    if (disabled) return;
    const current = options.findIndex((option) => option.value === value);
    active = start ?? (current >= 0 && options[current]?.disabled !== true ? current : move(options, -1, 1));
    open = true;
    // Safari doesn't focus a clicked button; the keys work only while it has focus.
    button?.focus({ preventScroll: true });
  }

  function close(): void {
    open = false;
    active = -1;
    search = "";
  }

  function choose(index: number): void {
    const option = options[index];
    if (option?.disabled === true) return;
    close();
    if (option === undefined || option.value === value) return;
    value = option.value;
    onchange?.({ currentTarget: { value: option.value } });
  }

  function type(key: string): void {
    clearTimeout(searchTimer);
    search += key;
    searchTimer = setTimeout(() => (search = ""), 600);
    const found = typeahead(options, search, open ? active : options.findIndex((o) => o.value === value));
    if (!open) show(found >= 0 ? found : null);
    else if (found >= 0) active = found;
  }

  /** Handles a key while the list is open; returns false for keys the list doesn't use. */
  function listKey(event: KeyboardEvent): boolean {
    switch (event.key) {
      case "ArrowDown":
        active = move(options, active, 1);
        return true;
      case "ArrowUp":
        if (event.altKey) choose(active);
        else active = move(options, active, -1);
        return true;
      case "Home":
        active = move(options, -1, 1);
        return true;
      case "End":
        active = move(options, options.length, -1);
        return true;
      case "PageDown":
        active = move(options, active, 1, 10);
        return true;
      case "PageUp":
        active = move(options, active, -1, 10);
        return true;
      case "Enter":
      case " ":
        choose(active);
        return true;
      case "Escape":
        // Keep a dialog or sheet around the select open.
        event.stopPropagation();
        close();
        return true;
      default:
        return false;
    }
  }

  function onkeydown(event: KeyboardEvent): void {
    const { key, altKey, ctrlKey, metaKey } = event;
    if (key === "Tab") {
      if (open) close();
      return;
    }
    if (key.length === 1 && !altKey && !ctrlKey && !metaKey && (key !== " " || search !== "")) {
      event.preventDefault();
      type(key);
      return;
    }
    if (open) {
      if (listKey(event)) event.preventDefault();
      return;
    }
    if (key === "ArrowDown" || key === "ArrowUp" || key === "Enter" || key === " ") show();
    else if (key === "Home") show(move(options, -1, 1));
    else if (key === "End") show(move(options, options.length, -1));
    else return;
    event.preventDefault();
  }

  const triggerProps: SelectTriggerProps = {
    get id() {
      return id;
    },
    type: "button",
    role: "combobox",
    "aria-haspopup": "listbox",
    get "aria-expanded"() {
      return open;
    },
    "aria-controls": listId,
    "aria-labelledby": labelId,
    get "aria-activedescendant"() {
      return open && active >= 0 ? optionId(active) : undefined;
    },
    get disabled() {
      return disabled === true;
    },
    onclick: () => {
      if (open) close();
      else show();
    },
    onkeydown,
    // Firefox clicks a button on Space's keyup even when its keydown was handled.
    onkeyup: (event) => {
      if (event.key === " ") event.preventDefault();
    },
    // Focus moving to another control closes the list; the window losing focus doesn't.
    onblur: (event) => {
      const next = event.relatedTarget as Node | null;
      if (open && next !== null && !(panel?.contains(next) ?? false)) close();
    },
    [createAttachmentKey()]: (node: HTMLElement) => {
      button = node;
      return () => {
        if (button === node) button = undefined;
      };
    },
  };

  $effect(() => {
    if (!open || button === undefined || panel === undefined) return;
    const anchor = button;
    const panelElement = panel;
    showPanel(panelElement);
    const stop = untrack(() => follow(anchor, panelElement, { placement, matchWidth: true, offset: 4 }));
    const onPointerDown = (event: PointerEvent): void => {
      const target = event.target as Node;
      if (!anchor.contains(target) && !panelElement.contains(target)) close();
    };
    document.addEventListener("pointerdown", onPointerDown, true);
    return () => {
      stop();
      hidePanel(panelElement);
      document.removeEventListener("pointerdown", onPointerDown, true);
    };
  });

  // Keep the active option in view: centered when the list opens, the nearest edge after that.
  let revealed = false;
  $effect(() => {
    if (!open || list === undefined) {
      revealed = false;
      return;
    }
    const row = active >= 0 ? list.querySelector<HTMLElement>(`#${CSS.escape(optionId(active))}`) : null;
    if (row === null) return;
    const box = list;
    if (!revealed) {
      box.scrollTop = row.offsetTop - (box.clientHeight - row.offsetHeight) / 2;
      revealed = true;
    } else if (row.offsetTop < box.scrollTop) {
      // The first and last rows bring the list's padding with them.
      box.scrollTop = row.previousElementSibling === null ? 0 : row.offsetTop;
    } else if (row.offsetTop + row.offsetHeight > box.scrollTop + box.clientHeight) {
      box.scrollTop =
        row.nextElementSibling === null ? box.scrollHeight : row.offsetTop + row.offsetHeight - box.clientHeight;
    }
  });
</script>

<div class={className}>
  <span id={labelId} class={hideLabel ? "sr-only" : "mb-1.5 block text-label-lg text-ink"}>{label}</span>
  {#if trigger}
    {@render trigger(triggerProps, shown)}
  {:else}
    <button
      {...rest}
      {...triggerProps}
      class="focus-field flex w-full items-center justify-between gap-2 rounded-field border bg-surface text-left text-body-2 disabled:border-line {variant ===
      'field'
        ? 'h-10 px-4'
        : 'h-9 pr-3 pl-3.5 max-sm:h-10'} {open
        ? 'border-accent shadow-[inset_0_0_0_1px_var(--color-accent)]'
        : 'border-line-strong hover:border-muted'}"
    >
      <span class="min-w-0 truncate {disabled ? 'text-faint' : value === '' ? 'text-muted' : 'text-ink'}">
        {shown}
      </span>
      <ChevronDown
        aria-hidden="true"
        class="size-5 shrink-0 transition-transform {open ? 'rotate-180' : ''} {disabled
          ? 'text-faint'
          : 'text-ink'}"
      />
    </button>
  {/if}
  {#if name !== undefined}
    <input type="hidden" {name} {value} />
  {/if}
  <!-- Pressing in the list keeps focus on the button, which handles the keys. -->
  <div
    bind:this={panel}
    popover="manual"
    data-floating
    role="presentation"
    onmousedown={(event) => event.preventDefault()}
    class="{panelSurface} max-w-[min(24rem,calc(100vw-1rem))] overflow-hidden"
  >
    <!-- The heights leave part of a row showing at the bottom of a long list (32px rows, 40px below lg), so it
         reads as one that scrolls. -->
    <ul
      bind:this={list}
      id={listId}
      role="listbox"
      aria-labelledby={labelId}
      tabindex="-1"
      class="scrollbar-thin relative max-h-[min(18rem,var(--floating-available))] overflow-y-auto overscroll-contain py-1.5 outline-none max-lg:max-h-[min(19.5rem,var(--floating-available))]"
    >
      {#if open}
        {#each options as option, index (option.value)}
          <!-- Keys are handled on the button (aria-activedescendant). -->
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <li
            id={optionId(index)}
            role="option"
            aria-selected={option.value === value}
            aria-disabled={option.disabled}
            data-active={index === active}
            class={listRow}
            onclick={() => choose(index)}
            onpointermove={() => {
              if (option.disabled !== true) active = index;
            }}
          >
            <span class="min-w-0 flex-1 break-words">{option.label}</span>
            <Check
              aria-hidden="true"
              class="size-4 shrink-0 text-accent-ink {option.value === value ? '' : 'invisible'}"
            />
          </li>
        {/each}
      {/if}
    </ul>
  </div>
</div>
