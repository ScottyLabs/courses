<!--
  Link previews for the whole site (Courseweave's CoursePreviewLayer, decision 41), mounted once in the root layout.
  Course links and elements marked with coursePreview() (../course/preview.ts) show a card about the course; links to
  instructor pages and names marked with instructorPreview() (../instructor/preview.ts) show one about the instructor,
  while someone is signed in (instructors are member data). Either card opens:
  - with a mouse, after resting on a link (neighbouring links open almost at once once a card has been open), and the
    card stays while the pointer moves onto it;
  - with the keyboard, when one is focused (focus-visible only); Escape closes it;
  - on a touch screen, by pressing and holding one: the card opens at the bottom of the screen with a link to the
    page, the press doesn't follow the link, and a tap anywhere else closes it. A plain tap still follows the link.
  ./preview-input.ts decides when; this component finds the anchors and shows the card.
-->
<script lang="ts">
  import { ArrowRight, X } from "@lucide/svelte";
  import { afterNavigate } from "$app/navigation";
  import Button from "$lib/components/ui/Button.svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import { follow, hidePanel, showPanel } from "$lib/components/ui/floating";
  import { coursePath, instructorPath } from "$lib/core/routes";
  import { session } from "$lib/data";
  import CoursePreviewCard from "../course/CoursePreviewCard.svelte";
  import {
    linkTarget,
    markedTarget,
    pageCourse,
    PREVIEW_ATTR,
    PREVIEW_TERM_ATTR,
    type PreviewTarget,
  } from "../course/preview";
  import InstructorPreviewCard from "../instructor/InstructorPreviewCard.svelte";
  import {
    instructorLinkTarget,
    INSTRUCTOR_PREVIEW_ATTR,
    INSTRUCTOR_TERM_ATTR,
    markedInstructor,
    pageInstructor,
    type InstructorTarget,
  } from "../instructor/preview";
  import { PreviewInput, type PreviewAnchor, type PreviewMode } from "./preview-input";

  type Target = ({ kind: "course" } & PreviewTarget) | ({ kind: "instructor" } & InstructorTarget);

  const CARD = "data-preview-card";
  const ANCHORS = `[${PREVIEW_ATTR}], [${INSTRUCTOR_PREVIEW_ATTR}], a[href]`;
  const uid = $props.id();
  const describedBy = `link-preview-${uid}`;
  let active = $state<(PreviewAnchor<Target> & { mode: PreviewMode }) | null>(null);
  let card = $state<HTMLElement>();
  const noun = $derived(active?.target.kind === "instructor" ? "instructor" : "course");
  /** Where the instructor card says "Open instructor" should go, when not where the link goes. */
  let openHref = $state<string | null>(null);

  function inCard(node: EventTarget | null): boolean {
    return node instanceof Element && node.closest(`[${CARD}]`) !== null;
  }

  function course(target: PreviewTarget | null): Target | null {
    if (target === null || target.courseId === pageCourse(location.pathname, location.search)) return null;
    return { kind: "course", ...target };
  }

  // Instructors are member data: signed out, nothing links to them, and a card would only say so.
  function instructor(target: InstructorTarget | null): Target | null {
    if (target === null || !session.signedIn || target.slug === pageInstructor(location.pathname)) return null;
    return { kind: "instructor", ...target };
  }

  /**
   * What an anchor previews. A mark comes first: an element marked for a course or an instructor previews it, and a
   * link marked "off" previews nothing of that kind. Otherwise a link previews the course or instructor it opens.
   */
  function targetOf(anchor: Element): Target | null {
    const courseMark = anchor.getAttribute(PREVIEW_ATTR);
    const instructorMark = anchor.getAttribute(INSTRUCTOR_PREVIEW_ATTR);
    if (courseMark !== null && courseMark !== "off")
      return course(markedTarget(courseMark, anchor.getAttribute(PREVIEW_TERM_ATTR)));
    if (instructorMark !== null && instructorMark !== "off")
      return instructor(markedInstructor(instructorMark, anchor.getAttribute(INSTRUCTOR_TERM_ATTR)));
    const link = anchor.getAttribute("href");
    if (link === null) return null;
    if (courseMark === null) {
      const found = linkTarget(link, location.origin);
      if (found !== null) return course(found);
    }
    return instructorMark === null ? instructor(instructorLinkTarget(link, location.origin)) : null;
  }

  /** The link or marked element at `node`, and what it previews; null when there's none to show. */
  function find(node: EventTarget | null): PreviewAnchor<Target> | null {
    if (!(node instanceof Element) || inCard(node)) return null;
    const anchor = node.closest(ANCHORS);
    if (anchor === null) return null;
    const target = targetOf(anchor);
    if (target === null) return null;
    let href: string;
    if (anchor instanceof HTMLAnchorElement) href = anchor.href;
    else if (target.kind === "course") href = coursePath(target.courseId, target.termId);
    else href = instructorPath(target.slug, target.termId);
    return { anchor, target, href };
  }

  /** Shows the card inside the anchor's open dialog, if any, so a modal dialog doesn't make it inert. */
  function host(node: HTMLElement): void {
    const dialog = active?.anchor.closest("dialog[open]");
    if (dialog !== null && dialog !== undefined && node.parentElement !== dialog) dialog.append(node);
  }

  $effect(() =>
    new PreviewInput<Target>({
      find,
      inCard,
      current: () => active,
      open: (found, mode) => {
        openHref = null;
        active = { ...found, mode };
      },
      close: () => {
        active = null;
      },
    }).install(),
  );

  // Signing out hides an instructor's card with the rest of the member data.
  $effect(() => {
    if (!session.signedIn && active?.target.kind === "instructor") active = null;
  });

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

{#snippet body(target: Target, id: string | undefined, room: boolean)}
  {#if target.kind === "course"}
    <CoursePreviewCard {id} courseId={target.courseId} termId={target.termId} {room} />
  {:else}
    <InstructorPreviewCard
      {id}
      slug={target.slug}
      name={target.name}
      termId={target.termId}
      {room}
      bind:openHref
    />
  {/if}
{/snippet}

{#if active}
  {#key active}
    {#if active.mode === "touch"}
      <div
        bind:this={card}
        {@attach host}
        popover="manual"
        data-floating
        data-preview-card=""
        role="dialog"
        aria-label="{noun === 'course' ? 'Course' : 'Instructor'} preview"
        style="position: fixed; left: 8px; right: 8px; top: auto; bottom: calc(8px + env(safe-area-inset-bottom)); margin: 0 auto; max-width: 28rem;"
        class="max-h-[70dvh] overflow-y-auto rounded-card border border-line bg-surface p-4 text-ink shadow-dialog"
      >
        <div class="absolute top-2 right-2">
          <IconButton label="Close preview" icon={X} onclick={() => (active = null)} />
        </div>
        {@render body(active.target, undefined, true)}
        <div class="mt-3 flex justify-end">
          <Button variant="primary" trailingIcon={ArrowRight} href={openHref ?? active.href}>Open {noun}</Button>
        </div>
      </div>
    {:else}
      <div
        bind:this={card}
        {@attach host}
        popover="manual"
        data-floating
        data-preview-card=""
        role="tooltip"
        style="width: min(20rem, calc(100vw - 16px));"
        class="rounded-card border border-line bg-surface p-3.5 text-ink shadow-popover"
      >
        {@render body(active.target, describedBy, false)}
      </div>
    {/if}
  {/key}
{/if}
