<script lang="ts" module>
  /** Spread these onto the element the tooltip describes. */
  export interface TooltipTriggerProps {
    "aria-describedby": string;
    onpointerenter: () => void;
    onpointerleave: () => void;
    onfocus: () => void;
    onblur: () => void;
  }
</script>

<script lang="ts">
  import { untrack, type Snippet } from "svelte";
  import { follow, hidePanel, showPanel, type Placement } from "./floating";

  let {
    text,
    placement = "top",
    delay = 350,
    trigger,
  }: {
    text: string;
    placement?: Placement;
    /** Hover delay in milliseconds; keyboard focus shows it at once. */
    delay?: number;
    trigger: Snippet<[TooltipTriggerProps]>;
  } = $props();

  const id = $props.id();
  let open = $state(false);
  let anchor = $state<HTMLElement>();
  let panel = $state<HTMLElement>();
  let timer: ReturnType<typeof setTimeout> | undefined;

  function show(wait: number): void {
    clearTimeout(timer);
    timer = setTimeout(() => {
      open = true;
    }, wait);
  }

  function hide(): void {
    clearTimeout(timer);
    open = false;
  }

  const triggerProps: TooltipTriggerProps = {
    "aria-describedby": id,
    onpointerenter: () => show(delay),
    onpointerleave: hide,
    onfocus: () => show(0),
    onblur: hide,
  };

  $effect(() => {
    if (!open || anchor === undefined || panel === undefined) return;
    const panelElement = panel;
    showPanel(panelElement);
    const stop = untrack(() => follow(anchor as HTMLElement, panelElement, { placement, offset: 6 }));
    const onKeyDown = (event: KeyboardEvent): void => {
      if (event.key === "Escape") hide();
    };
    document.addEventListener("keydown", onKeyDown);
    return () => {
      stop();
      hidePanel(panelElement);
      document.removeEventListener("keydown", onKeyDown);
    };
  });
</script>

<span bind:this={anchor} class="inline-flex">{@render trigger(triggerProps)}</span>
<div
  bind:this={panel}
  {id}
  popover="manual"
  data-floating
  role="tooltip"
  class="pointer-events-none max-w-64 rounded-[6px] bg-strong px-2 py-1 text-label-md text-surface shadow-popover"
>
  {text}
</div>
