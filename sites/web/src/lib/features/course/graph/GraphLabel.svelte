<script lang="ts">
  // A bracket's label on the requisite graph: "all of", "one of", "2 of", "with", "requires" or "unlocks" in the group's state (an
  // icon and a color), or a department's name and count. Drawn only: each course's accessible name says its group.
  import type { Placed } from "./geometry";
  import { stateLook } from "./looks";
  import { STATE_WORDS } from "./words";

  let { item }: { item: Extract<Placed, { kind: "label" }> } = $props();

  const node = $derived(item.node);
  const look = $derived(stateLook(node.state));
  const title = $derived(node.state === undefined ? node.title : `${node.title}: ${STATE_WORDS[node.state]}`);
</script>

{#if node.op === "dept"}
  <!-- As wide as its words (the line runs on under it), so it doesn't hide a stretch of the line for nothing. -->
  <span
    aria-hidden="true"
    class="absolute flex items-center gap-1.5 bg-surface pr-1.5 text-label-md text-ink"
    style="left:{item.x}px;top:{item.y}px;height:{item.h}px;max-width:{item.w}px"
    {title}><span class="truncate">{node.label}</span><span class="shrink-0 text-muted tabular">{node.count}</span></span
  >
{:else}
  <span
    aria-hidden="true"
    class="absolute flex items-center justify-center gap-1 rounded-full border bg-surface text-label-md font-semibold whitespace-nowrap {look.label}"
    style="left:{item.x}px;top:{item.y}px;width:{item.w}px;height:{item.h}px"
    {title}
  >
    {#if look.icon}<look.icon class="size-3 shrink-0" strokeWidth={3} />{/if}{node.label}{#if node.count !== undefined}<span
        class="font-medium text-muted tabular">{node.count}</span
      >{/if}
  </span>
{/if}
