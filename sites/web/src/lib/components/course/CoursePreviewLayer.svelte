<!--
  Course previews for the whole site (Courseweave's CoursePreviewLayer), mounted once in the root layout. Course links
  and elements marked with coursePreview() (./preview.ts) show a card about the course:
  - with a mouse, after resting on one (neighbouring codes open almost at once once a card has been open), and the
    card stays while the pointer moves onto it;
  - with the keyboard, when one is focused (focus-visible only); Escape closes it;
  - on a touch screen, by pressing and holding one: the card opens at the bottom of the screen with a link to the
    course, the press doesn't follow the link, and a tap anywhere else closes it. A plain tap still follows the link.
  ./preview-input.ts decides when; this component finds the anchors and shows the card.
-->
<script lang="ts">
  import { ArrowRight, X } from "@lucide/svelte";
  import { afterNavigate } from "$app/navigation";
  import Button from "$lib/components/ui/Button.svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import { follow, hidePanel, showPanel } from "$lib/components/ui/floating";
  import { coursePath } from "$lib/core/routes";
  import CoursePreviewCard from "./CoursePreviewCard.svelte";
  import { linkTarget, markedTarget, pageCourse, PREVIEW_ATTR, PREVIEW_TERM_ATTR } from "./preview";
  import { PreviewInput, type PreviewAnchor, type PreviewMode } from "./preview-input";

  const CARD = "data-course-preview-card";
  const uid = $props.id();
  const describedBy = `course-preview-${uid}`;
  let active = $state<(PreviewAnchor & { mode: PreviewMode }) | null>(null);
  let card = $state<HTMLElement>();

  function inCard(node: EventTarget | null): boolean {
    return node instanceof Element && node.closest(`[${CARD}]`) !== null;
  }

  /** The course link or marked element at `node`, and what it previews; null when there's none to show. */
  function find(node: EventTarget | null): PreviewAnchor | null {
    if (!(node instanceof Element) || inCard(node)) return null;
    const anchor = node.closest(`[${PREVIEW_ATTR}], a[href]`);
    if (anchor === null) return null;
    const mark = anchor.getAttribute(PREVIEW_ATTR);
    const link = anchor.getAttribute("href");
    let target = null;
    if (mark !== null) target = markedTarget(mark, anchor.getAttribute(PREVIEW_TERM_ATTR));
    else if (link !== null) target = linkTarget(link, location.origin);
    if (target === null || target.courseId === pageCourse(location.pathname, location.search)) return null;
    const href = anchor instanceof HTMLAnchorElement ? anchor.href : coursePath(target.courseId, target.termId);
    return { anchor, target, href };
  }

  /** Shows the card inside the anchor's open dialog, if any, so a modal dialog doesn't make it inert. */
  function host(node: HTMLElement): void {
    const dialog = active?.anchor.closest("dialog[open]");
    if (dialog !== null && dialog !== undefined && node.parentElement !== dialog) dialog.append(node);
  }

  $effect(() =>
    new PreviewInput({
      find,
      inCard,
      current: () => active,
      open: (found, mode) => {
        active = { ...found, mode };
      },
      close: () => {
        active = null;
      },
    }).install(),
  );

  // The card: in the top layer, placed by its anchor (hover) or at the bottom of the screen (touch).
  $effect(() => {
    const current = active;
    const panel = card;
    if (current === null || panel === undefined) return;
    showPanel(panel);
    const stop =
      current.mode === "hover" ? follow(current.anchor, panel, { placement: "bottom-start", offset: 8 }) : null;
    // While the card describes its anchor, screen readers hear it as the anchor's description.
    const previous = current.anchor.getAttribute("aria-describedby");
    if (current.mode === "hover")
      current.anchor.setAttribute("aria-describedby", previous === null ? describedBy : `${previous} ${describedBy}`);
    // The anchor can disappear (a list re-renders, a dialog closes); drop the card with it.
    const watch = setInterval(() => {
      if (!current.anchor.isConnected) active = null;
    }, 300);
    return () => {
      stop?.();
      clearInterval(watch);
      hidePanel(panel);
      if (current.mode !== "hover") return;
      if (previous === null) current.anchor.removeAttribute("aria-describedby");
      else current.anchor.setAttribute("aria-describedby", previous);
    };
  });

  afterNavigate(() => {
    active = null;
  });
</script>

{#if active}
  {#key active}
    {#if active.mode === "touch"}
      <div
        bind:this={card}
        {@attach host}
        popover="manual"
        data-floating
        data-course-preview-card=""
        role="dialog"
        aria-label="Course preview"
        style="position: fixed; left: 8px; right: 8px; top: auto; bottom: calc(8px + env(safe-area-inset-bottom)); margin: 0 auto; max-width: 28rem;"
        class="max-h-[70dvh] overflow-y-auto rounded-card border border-line bg-surface p-4 text-ink shadow-dialog"
      >
        <div class="absolute top-2 right-2">
          <IconButton label="Close preview" icon={X} onclick={() => (active = null)} />
        </div>
        <CoursePreviewCard courseId={active.target.courseId} termId={active.target.termId} room />
        <div class="mt-3 flex justify-end">
          <Button variant="primary" trailingIcon={ArrowRight} href={active.href}>Open course</Button>
        </div>
      </div>
    {:else}
      <div
        bind:this={card}
        {@attach host}
        popover="manual"
        data-floating
        data-course-preview-card=""
        role="tooltip"
        style="width: min(20rem, calc(100vw - 16px));"
        class="rounded-card border border-line bg-surface p-3.5 text-ink shadow-popover"
      >
        <CoursePreviewCard id={describedBy} courseId={active.target.courseId} termId={active.target.termId} />
      </div>
    {/if}
  {/key}
{/if}
