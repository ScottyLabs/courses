<!--
  The requisite explorer's graph view: the course in the middle, what it requires to its left (brackets labelled AND,
  OR, "2 OF" and, dashed, WITH for corequisites) and what it unlocks to its right, by department when there are many;
  on phones, top to bottom. Courses and brackets show the student's standing from the course pages' own checks.
  Clicking a course expands its requisites, previews it, opens it or centers the graph on it ("On click").
  ./model.ts builds it, ./layout.ts places it, ./pan-zoom.svelte.ts pans and zooms it.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { termRequisites } from "$lib/components/course/requisite-context.svelte";
  import type { Unlock } from "$lib/core/catalog";
  import type { CatalogIndex } from "$lib/core/catalog-types";
  import type { Department } from "$lib/core/data";
  import { currentTermCode } from "$lib/core/plan";
  import { codeOf } from "$lib/core/requisite-format";
  import { evaluateRequisite } from "$lib/core/requisites";
  import { coursePath } from "$lib/core/routes";
  import { record } from "$lib/state";
  import { departmentOptions, filterUnlocks, NO_FILTER, prefixNamer, type UnlockFilter } from "./filters";
  import { DESKTOP_METRICS, PHONE_METRICS, type Placed, type Rect } from "./geometry";
  import GraphCanvas from "./GraphCanvas.svelte";
  import GraphLegend from "./GraphLegend.svelte";
  import GraphPill from "./GraphPill.svelte";
  import GraphRoot from "./GraphRoot.svelte";
  import GraphStatusLine from "./GraphStatusLine.svelte";
  import GraphToolbar from "./GraphToolbar.svelte";
  import { layoutGraph } from "./layout";
  import { buildGraph, parentsOf, walkGraph } from "./model";
  import { PanZoom } from "./pan-zoom.svelte";
  import PinnedPreview from "./PinnedPreview.svelte";
  import { storedClickAction, storedShowUnlocks, type ClickAction } from "./settings";
  import { courseStatus, graphSummary } from "./status";
  import type { CourseNode } from "./types";
  import UnlockFilters from "./UnlockFilters.svelte";
  import { couldTakeNext, withCourseDone } from "./unlocks";
  import { courseLabel } from "./words";

  let {
    courseId,
    index,
    unlocks,
    termId,
    termName,
    offered,
    departments = [],
    title,
    centerHref,
  }: {
    /** The course in the middle, as a five-digit id. */
    courseId: string;
    index: CatalogIndex;
    /** From buildUnlocks(index).get(courseId). */
    unlocks: readonly Unlock[];
    /** The semester requisites are checked for; null shows plain courses. */
    termId: string | null;
    termName?: string;
    /** Whether the semester offers a course; omit to hide that filter. */
    offered?: (courseId: string) => boolean;
    /** The semester's departments, to name course-number prefixes. */
    departments?: readonly Department[];
    title: (courseId: string) => string | undefined;
    /** The explorer centered on a course, in this view. */
    centerHref: (courseId: string) => string;
  } = $props();

  /** "Expand all" opens this many levels; deeper courses keep their + button. */
  const AUTO_DEPTH = 3;
  /** Unlock filters show from this many unlocks. */
  const FILTER_FROM = 13;
  const uid = $props.id();
  const currentTerm = currentTermCode();

  // The student's standing, as the course pages' status line has it: with nothing on record, "missing" reads as "not
  // entered yet".
  const ctx = $derived(termId === null ? null : termRequisites(termId).ctx);
  const summary = $derived(ctx === null ? null : graphSummary(courseId, index.courses[courseId], ctx));
  const own = $derived.by(() => {
    if (ctx === null) return null;
    const result = evaluateRequisite({ type: "course", id: courseId }, ctx, { allowConcurrent: true });
    return result.status === "met" || result.status === "planned" ? courseStatus(result, currentTerm) : null;
  });
  const unchecked = $derived(
    ctx === null ||
      ctx.credits.size === 0 ||
      (record.isEmpty && summary?.level === "problem" && summary.check.antireqs.length === 0),
  );

  let clickAction = $state<ClickAction>(storedClickAction());
  let showUnlocks = $state(storedShowUnlocks());
  let expanded = $state<ReadonlySet<string>>(new Set());
  let collapsed = $state<ReadonlySet<string>>(new Set());
  let autoDepth = $state(0);
  let revealed = $state<ReadonlySet<string>>(new Set());
  let filter = $state<UnlockFilter>({ ...NO_FILTER });
  let pinnedKey = $state<string | null>(null);

  const nameOf = $derived(prefixNamer(departments));
  const readyIds = $derived.by(() => {
    if (ctx === null || record.isEmpty) return new Set<string>();
    const done = withCourseDone(ctx, courseId);
    return new Set(unlocks.filter((u) => couldTakeNext(index, u.id, ctx, done)).map((u) => u.id));
  });
  const shownUnlocks = $derived(filterUnlocks(unlocks, filter, offered, readyIds));
  const model = $derived(
    buildGraph({
      index,
      rootId: courseId,
      ctx,
      unchecked: record.isEmpty,
      currentTerm,
      expanded,
      collapsed,
      autoDepth,
      revealed,
      unlocks: showUnlocks ? shownUnlocks : null,
      departmentOf: nameOf,
    }),
  );
  const parents = $derived(parentsOf(model));
  const nodes = $derived([...walkGraph(model)]);

  let wrapWidth = $state(0);
  let innerHeight = $state(800);
  const phone = $derived(wrapWidth > 0 && wrapWidth < 640);
  const layout = $derived(layoutGraph(model, phone ? PHONE_METRICS : DESKTOP_METRICS, phone ? "outline" : "horizontal"));
  const panZoom = new PanZoom({
    content: () => ({ width: layout.width, height: layout.height }),
    home: () => ({ x: layout.root.x + layout.root.w / 2, y: layout.root.y + layout.root.h / 2 }),
    centered: () => !phone,
    floor: () => (phone ? 0.55 : 0.7),
  });
  // Wide screens: as tall as the graph at its first zoom, within the window. Phones: as tall as the graph, so the
  // page scrolls it and up-and-down swipes never get caught in a box.
  const viewHeight = $derived(
    phone
      ? Math.ceil(layout.height * panZoom.zoom)
      : Math.round(Math.min(Math.max(layout.height * panZoom.base, 240), Math.max(420, innerHeight * 0.72))),
  );
  /** Just the course: nothing to zoom, nothing for a legend to explain. */
  const lonely = $derived(layout.items.length === 0);

  // A new course, layout or unlocks shown or hidden starts from the first view; until the student moves it, the view
  // follows the graph as statuses and unlocks load.
  let viewKey = "";
  $effect(() => {
    const key = `${courseId}|${phone}|${showUnlocks}`;
    if (panZoom.boxWidth === 0 || key === viewKey) return;
    viewKey = key;
    untrack(() => panZoom.reset());
  });
  $effect(() => {
    void layout;
    if (panZoom.pristine && panZoom.boxWidth > 0) untrack(() => panZoom.home());
  });
  $effect(() => {
    void clickAction;
    untrack(() => (pinnedKey = null));
  });

  const rectOf = (key: string): Rect | null =>
    key === "root" ? layout.root : (layout.items.find((i) => `${i.kind}:${i.node.key}` === key) ?? null);
  const add = (set: ReadonlySet<string>, key: string): ReadonlySet<string> => new Set([...set, key]);
  const without = (set: ReadonlySet<string>, key: string): ReadonlySet<string> =>
    new Set([...set].filter((k) => k !== key));

  function toggle(node: CourseNode): void {
    panZoom.keep(
      () => rectOf(`course:${node.key}`),
      () => {
        expanded = node.expanded ? without(expanded, node.key) : add(expanded, node.key);
        collapsed = node.expanded ? add(collapsed, node.key) : without(collapsed, node.key);
      },
    );
  }

  function toggleAll(): void {
    panZoom.keep(
      () => rectOf("root"),
      () => {
        autoDepth = autoDepth > 0 ? 0 : AUTO_DEPTH;
        expanded = new Set();
        collapsed = new Set();
      },
    );
  }

  function toggleMore(group: string): void {
    // The group's first item stays put: it shows either way.
    const first = nodes.find((n) => n.kind === "group" && n.key === group);
    const item = first?.kind === "group" ? first.items[0] : undefined;
    const key = item === undefined ? "root" : `${item.kind === "group" ? "label" : item.kind}:${item.key}`;
    panZoom.keep(
      () => rectOf(key),
      () => (revealed = revealed.has(group) ? without(revealed, group) : add(revealed, group)),
    );
  }

  const linkTerm = (id: string): string | null => (offered === undefined || offered(id) ? termId : null);

  /** Where clicking a course goes, or null when it acts in place (expands or previews it). */
  function hrefOf(node: CourseNode, side: Placed["side"]): string | null {
    if (clickAction === "open") return coursePath(node.id, linkTerm(node.id));
    // What it unlocks has nothing to expand: following it centers the graph on it.
    if (clickAction === "center" || (clickAction === "expand" && side === "right")) return centerHref(node.id);
    return null;
  }

  function act(node: CourseNode): void {
    if (clickAction === "expand" && node.expandable) toggle(node);
    else pinnedKey = pinnedKey === node.key ? null : node.key;
  }

  let section = $state<HTMLElement>();
  const pinned = $derived(nodes.find((n): n is CourseNode => n.kind === "course" && n.key === pinnedKey) ?? null);
  const anchor = $derived(
    pinned === null ? null : (section?.querySelector<HTMLElement>(`[data-graph-key="course:${CSS.escape(pinned.key)}"]`) ?? null),
  );

  function togglePinned(): void {
    if (pinned !== null) toggle(pinned);
  }

  function closePin(focusAnchor: boolean): void {
    if (focusAnchor) anchor?.focus({ preventScroll: true });
    pinnedKey = null;
  }
</script>

<svelte:window bind:innerHeight />

<section
  bind:this={section}
  aria-label="Requisite graph"
  class="min-w-0 rounded-card border border-pressed bg-surface"
  bind:clientWidth={wrapWidth}
>
  <GraphToolbar
    bind:clickAction
    bind:showUnlocks
    hasUnlocks={unlocks.length > 0}
    canExpand={nodes.some((n) => n.kind === "course" && n.expandable)}
    expandedAll={autoDepth > 0}
    depth={AUTO_DEPTH}
    ontoggleall={toggleAll}
    {panZoom}
    zoomable={!lonely}
  />
  {#if showUnlocks && unlocks.length >= FILTER_FROM}
    <UnlockFilters
      bind:requiredOnly={filter.requiredOnly}
      bind:offeredOnly={filter.offeredOnly}
      bind:readyOnly={filter.readyOnly}
      bind:department={filter.department}
      termName={offered === undefined ? undefined : termName}
      canReady={readyIds.size > 0}
      departments={departmentOptions(unlocks, nameOf)}
      shown={shownUnlocks.length}
      total={unlocks.length}
    />
  {/if}
  <GraphStatusLine {courseId} {summary} {unchecked} loading={termId !== null} unlocksNothing={unlocks.length === 0} />

  <GraphCanvas
    {layout}
    {panZoom}
    height={viewHeight}
    label="Requisites of {codeOf(courseId)}"
    describedBy="{uid}-keys"
    ontoggle={toggle}
    onmore={toggleMore}
  >
    {#snippet root(tabindex: number)}
      <GraphRoot
        rect={layout.root}
        {courseId}
        title={title(courseId)}
        href={coursePath(courseId, linkTerm(courseId))}
        {own}
        {summary}
        {unchecked}
        {tabindex}
      />
    {/snippet}
    {#snippet course(item: Extract<Placed, { kind: "course" }>, tabindex: number)}
      <GraphPill
        {item}
        href={hrefOf(item.node, item.side)}
        label={courseLabel(item.node, item.side, title(item.node.id), parents.get(item.node.key))}
        {tabindex}
        expands={clickAction === "expand" && item.node.expandable}
        pinned={pinnedKey === item.node.key}
        previewTerm={linkTerm(item.node.id)}
        onact={() => act(item.node)}
      />
    {/snippet}
  </GraphCanvas>

  <div class="flex flex-col gap-2 border-t border-line px-4 py-3 {lonely ? 'hidden' : ''}">
    <GraphLegend {unchecked} unlocks={showUnlocks && unlocks.length > 0} />
    <p id="{uid}-keys" class="text-label-md text-muted">
      Drag or scroll to move around; ctrl and scroll, or pinch, to zoom. With the keyboard, the arrow keys move between
      courses, Enter does what a click does, and + and &minus; zoom.
    </p>
  </div>
</section>

{#if pinned !== null && anchor !== null}
  {#key pinned.key}
    <PinnedPreview
      node={pinned}
      {anchor}
      termId={linkTerm(pinned.id)}
      {centerHref}
      ontoggle={togglePinned}
      onclose={closePin}
    />
  {/key}
{/if}
