<script lang="ts" module>
  /** Spread these onto the element that opens the popover. */
  export interface PopoverTriggerProps {
    "aria-expanded": boolean;
    "aria-controls": string;
    "aria-haspopup": "dialog" | "menu" | "listbox";
    onclick: (event: MouseEvent) => void;
    onkeydown: (event: KeyboardEvent) => void;
  }
</script>

<script lang="ts">
  import { untrack, type Snippet } from "svelte";
  import { follow, hidePanel, showPanel, type Placement } from "./floating";
  import { focusableWithin, restoreFocus } from "./focus";
  import { panelSurface } from "./styles";

  let {
    open = $bindable(false),
    placement = "bottom-start",
    offset = 6,
    matchWidth = false,
    label,
    role = "dialog",
    focusOnOpen = true,
    trigger,
    children,
    ontriggerkeydown,
    onclose,
    class: className = "",
  }: {
    open?: boolean;
    placement?: Placement;
    offset?: number;
    /** Make the panel at least as wide as the trigger. */
    matchWidth?: boolean;
    /** Accessible name for the panel. */
    label?: string;
    role?: "dialog" | "menu" | "listbox";
    /** Move focus to the first focusable element in the panel when it opens. */
    focusOnOpen?: boolean;
    /** Renders the trigger; spread the props onto a button. */
    trigger: Snippet<[PopoverTriggerProps]>;
    /** Panel content. `close(false)` closes without returning focus to the trigger. */
    children: Snippet<[{ close: (focusTrigger?: boolean) => void }]>;
    /** Extra key handling on the trigger (Menu opens on arrow keys). */
    ontriggerkeydown?: (event: KeyboardEvent) => void;
    onclose?: () => void;
    /** Classes for the panel. */
    class?: string;
  } = $props();

  const panelId = $props.id();
  let anchor = $state<HTMLElement>();
  let panel = $state<HTMLElement>();
  let returnFocus = true;

  function close(focusTrigger = true): void {
    returnFocus = focusTrigger;
    open = false;
  }

  function triggerElement(): HTMLElement | null {
    return anchor?.querySelector<HTMLElement>("button, a[href], input, [tabindex]") ?? null;
  }

  const triggerProps: PopoverTriggerProps = {
    get "aria-expanded"() {
      return open;
    },
    "aria-controls": panelId,
    get "aria-haspopup"() {
      return role;
    },
    onclick: () => {
      open = !open;
    },
    onkeydown: (event) => ontriggerkeydown?.(event),
  };

  $effect(() => {
    if (!open || anchor === undefined || panel === undefined) return;
    const anchorElement = anchor;
    const panelElement = panel;
    returnFocus = true;
    showPanel(panelElement);
    const stop = untrack(() => follow(anchorElement, panelElement, { placement, offset, matchWidth }));
    if (untrack(() => focusOnOpen)) {
      requestAnimationFrame(() => {
        (focusableWithin(panelElement).at(0) ?? panelElement).focus({ preventScroll: true });
      });
    }
    const onPointerDown = (event: PointerEvent): void => {
      const target = event.target as Node;
      if (!anchorElement.contains(target) && !panelElement.contains(target)) close(false);
    };
    const onKeyDown = (event: KeyboardEvent): void => {
      if (event.key !== "Escape") return;
      const active = document.activeElement;
      if (panelElement.contains(active) || anchorElement.contains(active)) {
        event.preventDefault();
        event.stopPropagation();
        close(true);
      }
    };
    document.addEventListener("pointerdown", onPointerDown, true);
    document.addEventListener("keydown", onKeyDown, true);
    return () => {
      stop();
      document.removeEventListener("pointerdown", onPointerDown, true);
      document.removeEventListener("keydown", onKeyDown, true);
      hidePanel(panelElement);
      if (returnFocus) restoreFocus(triggerElement());
      onclose?.();
    };
  });

  function onfocusout(event: FocusEvent): void {
    const next = event.relatedTarget as Node | null;
    if (next !== null && !panel?.contains(next) && !anchor?.contains(next)) close(false);
  }
</script>

<span bind:this={anchor} class="inline-flex">{@render trigger(triggerProps)}</span>
<div
  bind:this={panel}
  id={panelId}
  popover="manual"
  data-floating
  {role}
  aria-label={label}
  tabindex="-1"
  {onfocusout}
  class="{panelSurface} outline-none {className}"
>
  {#if open}{@render children({ close })}{/if}
</div>
