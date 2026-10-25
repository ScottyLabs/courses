<script lang="ts">
  // A course's preview pinned open by clicking it on the requisite graph ("On click: Show course preview"): the site's
  // course preview card, the student's standing on it, and what to do next (show its prerequisites here, center the
  // graph on it, open it). Closes on Escape (back to the course), the close button or a click elsewhere.
  import { ArrowRight, X } from "@lucide/svelte";
  import CoursePreviewCard from "$lib/components/course/CoursePreviewCard.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import { follow, hidePanel, showPanel } from "$lib/components/ui/floating";
  import { focusableWithin } from "$lib/components/ui/focus";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import { codeOf } from "$lib/core/requisite-format";
  import { coursePath } from "$lib/core/routes";
  import { pillLook } from "./looks";
  import type { CourseNode } from "./types";

  let {
    node,
    anchor,
    termId,
    centerHref,
    ontoggle,
    onclose,
  }: {
    node: CourseNode;
    /** The course on the graph it opened from. */
    anchor: HTMLElement;
    /** The semester to describe it in; null for none in particular. */
    termId: string | null;
    centerHref: (courseId: string) => string;
    ontoggle: () => void;
    /** `focusAnchor`: return focus to the course (Escape, the close button). */
    onclose: (focusAnchor: boolean) => void;
  } = $props();

  let panel = $state<HTMLElement>();
  const look = $derived(pillLook(node.status, node.ready === true));

  $effect(() => {
    const el = panel;
    if (el === undefined) return;
    showPanel(el);
    const stop = follow(anchor, el, { placement: "bottom-start", offset: 8 });
    const frame = requestAnimationFrame(() => {
      (focusableWithin(el).at(0) ?? el).focus({ preventScroll: true });
    });
    const onDown = (event: PointerEvent): void => {
      const target = event.target as Node;
      if (!el.contains(target) && !anchor.contains(target)) onclose(false);
    };
    document.addEventListener("pointerdown", onDown, true);
    return () => {
      cancelAnimationFrame(frame);
      stop();
      document.removeEventListener("pointerdown", onDown, true);
      hidePanel(el);
    };
  });

  function onkeydown(event: KeyboardEvent): void {
    if (event.key !== "Escape") return;
    event.preventDefault();
    event.stopPropagation();
    onclose(true);
  }
</script>

<div
  bind:this={panel}
  popover="manual"
  data-floating
  data-preview-card=""
  role="dialog"
  aria-label="{codeOf(node.id)} preview"
  tabindex="-1"
  style="width: min(21rem, calc(100vw - 16px));"
  class="rounded-card border border-line bg-surface p-3.5 text-ink shadow-popover outline-none"
  {onkeydown}
>
  <div class="absolute top-2 right-2">
    <IconButton label="Close preview" icon={X} size="sm" onclick={() => onclose(true)} />
  </div>
  <CoursePreviewCard courseId={node.id} {termId} room />
  {#if look !== null && (node.note !== undefined || node.ready === true)}
    <p class="mt-2 flex items-center gap-1.5 text-label-md text-ink">
      <span aria-hidden="true" class="grid size-5 shrink-0 place-items-center rounded-tag border {look.box}">
        <look.icon class="size-3.5 {look.tint}" strokeWidth={2.5} />
      </span>
      {look.label}: {node.note ?? look.help}
    </p>
  {/if}
  <div class="mt-3 flex flex-wrap justify-end gap-2">
    {#if node.expandable}
      <Button size="sm" variant="secondary" class="max-md:h-10" onclick={ontoggle}>
        {node.expanded ? "Hide its prerequisites" : "Show its prerequisites"}
      </Button>
    {/if}
    <Button size="sm" variant="secondary" class="max-md:h-10" href={centerHref(node.id)}>Center here</Button>
    <Button size="sm" variant="primary" class="max-md:h-10" trailingIcon={ArrowRight} href={coursePath(node.id, termId)}
      >Open course</Button
    >
  </div>
</div>
