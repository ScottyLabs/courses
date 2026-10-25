<script lang="ts">
  // A course on the requisite graph: its code with the student's standing as a fill and an icon, and its minimum
  // grade. A link when clicking it goes somewhere (opens the course, centers the graph on it), else a button that
  // expands its requisites or previews it; either way hovering or focusing it shows the site's course preview.
  import { Repeat } from "@lucide/svelte";
  import { coursePreview } from "$lib/components/course/preview";
  import { codeOf } from "$lib/core/requisite-format";
  import type { Placed } from "./geometry";
  import { pillLook, PLAIN_BOX } from "./looks";

  let {
    item,
    href,
    label,
    tabindex,
    expands,
    pinned,
    previewTerm,
    onact,
  }: {
    item: Extract<Placed, { kind: "course" }>;
    /** Where clicking goes; null when clicking acts in place. */
    href: string | null;
    /** Its accessible name (./words.ts). */
    label: string;
    tabindex: number;
    /** Clicking expands or collapses its requisites (rather than previewing it). */
    expands: boolean;
    /** Its preview is pinned open. */
    pinned: boolean;
    /** The semester its hover preview describes. */
    previewTerm: string | null;
    onact: () => void;
  } = $props();

  const node = $derived(item.node);
  const look = $derived(pillLook(node.status, node.ready === true));
  const key = $derived(`course:${node.key}`);
  const style = $derived(`left:${item.x}px;top:${item.y}px;width:${item.w}px;height:${item.h}px`);
  const classes = $derived(
    [
      "absolute flex items-center gap-1.5 rounded-tag border px-2.5 text-label-lg text-strong",
      "transition-[filter,border-color] hover:border-line-strong hover:brightness-[0.97]",
      node.relation === "coreq" && item.side === "right" ? "border-dashed" : "",
      look?.box ?? PLAIN_BOX,
    ].join(" "),
  );
  // A pinned preview is already showing; the hover one would only cover it.
  const preview = $derived(pinned ? { "data-course-preview": "off" } : coursePreview(node.id, previewTerm));
</script>

{#snippet body()}
  {#if look}<look.icon aria-hidden="true" class="size-4 shrink-0 {look.tint}" strokeWidth={2.5} />{/if}
  <span class="truncate tabular">{codeOf(node.id)}</span>
  {#if node.cycle}<Repeat aria-hidden="true" class="size-3.5 shrink-0 text-muted" />{/if}
  {#if node.minGrade !== undefined}
    <span
      aria-hidden="true"
      class="ml-auto shrink-0 rounded-[3px] px-1 text-label-md {node.status === 'grade-low'
        ? 'bg-danger text-surface'
        : 'bg-hover text-ink'}">{node.minGrade}</span
    >
  {/if}
{/snippet}

{#if href !== null}
  <a {href} class={classes} {style} aria-label={label} data-graph-key={key} {tabindex}>{@render body()}</a>
{:else}
  <button
    type="button"
    class={classes}
    {style}
    aria-label={label}
    aria-expanded={expands ? node.expanded : pinned}
    aria-haspopup={expands ? undefined : "dialog"}
    data-graph-key={key}
    {tabindex}
    onclick={onact}
    {...preview}>{@render body()}</button
  >
{/if}
