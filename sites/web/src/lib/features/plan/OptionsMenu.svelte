<script lang="ts">
  // A "..." button that opens a small panel of actions: the plan's, a semester's or a course's options. Arrow keys
  // move between the actions; Escape closes it and returns to the button. Like Menu, but the panel can hold labels,
  // a checkbox or a units field as well as buttons.
  import { Ellipsis } from "@lucide/svelte";
  import type { Snippet } from "svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import Popover from "$lib/components/ui/Popover.svelte";

  let {
    label,
    size = "sm",
    variant = "ghost",
    class: className = "",
    children: content,
  }: {
    /** Names the button and the panel: "Options for 15-122". */
    label: string;
    size?: "sm" | "md";
    variant?: "ghost" | "outline";
    class?: string;
    children: Snippet<[close: () => void]>;
  } = $props();

  let open = $state(false);

  function items(panel: HTMLElement): HTMLElement[] {
    return [
      ...panel.querySelectorAll<HTMLElement>(
        "button:not(:disabled), a[href], input:not(:disabled), select:not(:disabled)",
      ),
    ];
  }

  function onkeydown(event: KeyboardEvent): void {
    if (!(event.currentTarget instanceof HTMLElement)) return;
    const all = items(event.currentTarget);
    const at = all.indexOf(document.activeElement as HTMLElement);
    let next: number;
    if (event.key === "ArrowDown") next = at + 1;
    else if (event.key === "ArrowUp") next = at < 0 ? -1 : at - 1;
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = -1;
    else return;
    // Number fields keep their arrow keys.
    if (document.activeElement instanceof HTMLInputElement && document.activeElement.type === "number")
      return;
    event.preventDefault();
    all.at(next % Math.max(all.length, 1))?.focus();
  }
</script>

<Popover
  bind:open
  {label}
  placement="bottom-end"
  class="scrollbar-thin max-h-[min(70dvh,480px)] w-64 max-w-[calc(100vw-1rem)] overflow-y-auto py-1.5"
>
  {#snippet trigger(props)}
    <IconButton {...props} {label} {size} {variant} icon={Ellipsis} class="max-md:size-10 {className}" />
  {/snippet}
  {#snippet children({ close })}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="flex flex-col" {onkeydown}>
      {@render content(() => close(true))}
    </div>
  {/snippet}
</Popover>
