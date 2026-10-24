<script lang="ts">
  // A degree plan: the header with the plan's programs and numbers, the record prompt, requisite problems, the
  // semester grid and, beside it on wide screens (a tab otherwise), the requirements. Loads what the plan needs and
  // provides it to every part of the page. Courseweave's PlanView in features/degree/PlanPage.tsx.
  import { Maximize2 } from "@lucide/svelte";
  import { untrack } from "svelte";
  import { MediaQuery } from "svelte/reactivity";
  import Button from "$lib/components/ui/Button.svelte";
  import Dialog from "$lib/components/ui/Dialog.svelte";
  import Tabs from "$lib/components/ui/Tabs.svelte";
  import { toast } from "$lib/components/ui/toast.svelte";
  import type { Term } from "$lib/core/data";
  import {
    courseHref,
    currentTerm,
    fceHours,
    planDerived,
    publishedTerms,
    scheduledCourses,
  } from "$lib/core/degree/hooks";
  import { suggestTerm, type Plan } from "$lib/core/plan";
  import { auditData } from "$lib/core/programs";
  import { courseLookup } from "$lib/core/record/hooks";
  import { codeOf } from "$lib/core/requisite-format";
  import { termLabel } from "$lib/core/requisites";
  import { coursePath, profilePath } from "$lib/core/routes";
  import type { TermIndexEntry } from "$lib/core/types";
  import { catalog, resource } from "$lib/data";
  import RecordPrompt from "$lib/features/record/RecordPrompt.svelte";
  import { links, preferences, record, schedules } from "$lib/state";
  import { setPlanPage, type DragItem } from "./context";
  import PlanHeader from "./PlanHeader.svelte";
  import RequirementsPanel from "./RequirementsPanel.svelte";
  import RequisiteProblems from "./RequisiteProblems.svelte";
  import SemesterGrid from "./SemesterGrid.svelte";

  let {
    plan,
    routed,
    terms,
  }: {
    plan: Plan;
    /** The semester being browsed, once loaded. */
    routed: Term | null;
    /** Every built semester. */
    terms: TermIndexEntry[];
  } = $props();

  const current = currentTerm();
  const compactQuery = new MediaQuery("(max-width: 767px)");
  const dragQuery = new MediaQuery("(pointer: fine) and (min-width: 768px)");
  const wideQuery = new MediaQuery("(min-width: 1180px)");

  const index = resource(() => catalog.catalogIndex());
  const offerings = resource(() => catalog.offerings());
  const fce = resource(() => catalog.fceSummary());
  const catalogIndex = $derived(index.value ?? null);
  const published = $derived(publishedTerms(terms));

  // Course ids on each plan semester's published schedule, for offering hints and suggestions.
  const termsKey = $derived(plan.terms.map((t) => t.term).join(","));
  const scheduledRes = resource(() =>
    scheduledCourses(termsKey === "" ? [] : termsKey.split(","), untrack(() => published), (id) =>
      catalog.term(id),
    ),
  );
  const scheduled = $derived(scheduledRes.value ?? new Map<string, Set<string>>());

  // The selected programs' requirements: loaded again only when the programs change.
  const programsKey = $derived(JSON.stringify(plan.programs));
  const versions = resource(() =>
    programsKey === "[]" ? null : catalog.auditVersions(untrack(() => plan.programs)),
  );

  const recordLike = $derived({ entries: record.entries, overrides: record.overrides });
  const hours = $derived(fceHours(fce.value));
  const view = $derived(
    planDerived({
      plan,
      schedules: schedules.items,
      record: recordLike,
      catalog: catalogIndex,
      term: routed,
      fceHours: hours,
      current,
    }),
  );
  const data = $derived.by(() => {
    const loaded = versions.value;
    if (view === null || plan.programs.length === 0 || loaded === undefined) return null;
    if (!plan.programs.every((p) => loaded.has(p.auditId))) return null;
    const result = auditData(view.auditInput, loaded, catalogIndex);
    return result.status === "ready" ? result.value : null;
  });
  const auditError = $derived.by(() => {
    if (versions.status === "error") return versions.error;
    const loaded = versions.value;
    if (view === null || loaded === undefined || data !== null) return null;
    const result = auditData(view.auditInput, loaded, catalogIndex);
    return result.status === "error" ? result.error : null;
  });

  const lookup = $derived(courseLookup(routed, catalogIndex));
  const routedHref = $derived(routed === null ? null : courseHref(routed, offerings.value));
  function href(id: string, term: string | null = null): string {
    return routedHref === null ? coursePath(id, term) : routedHref(id, term);
  }

  let openTerms = $state<Record<string, boolean>>({});
  let dragging = $state<DragItem | null>(null);
  let expanded = $state(false);
  let tab = $state("semesters");

  function isOpen(term: string): boolean {
    const col = view?.columns.find((c) => c.term === term);
    return openTerms[term] ?? col?.phase !== "past";
  }

  function jumpTo(planCourseId: string, term: string): void {
    openTerms = { ...openTerms, [term]: true };
    tab = "semesters";
    const fromDialog = expanded;
    expanded = false;
    // After the semesters render, and after the dialog has closed and handed focus back.
    setTimeout(
      () => {
        const el = document.getElementById(`plan-course-${planCourseId}`);
        el?.scrollIntoView({ behavior: "smooth", block: "center" });
        el?.focus({ preventScroll: true });
      },
      fromDialog ? 120 : 30,
    );
  }

  function addSuggested(courseId: string): void {
    if (view === null) return;
    const target = suggestTerm(
      view.resolved,
      courseId,
      view.catalog,
      view.current,
      view.unitsOf,
      (t) => scheduled.get(t),
      undefined,
      recordLike,
    );
    if (target === null) {
      toast("Add a semester to the plan first.", { kind: "error" });
      return;
    }
    const id = links.addToTerm(plan, target, courseId);
    toast(`Added ${codeOf(courseId)} to ${termLabel(target)}.`, { kind: "success" });
    jumpTo(id, target);
  }

  setPlanPage({
    get plan() {
      return plan;
    },
    get derived() {
      // PlanView renders only with a plan, so the derivation always exists here.
      // oxlint-disable-next-line typescript/no-non-null-assertion
      return view!;
    },
    get catalog() {
      return catalogIndex;
    },
    get routed() {
      return routed;
    },
    get record() {
      return recordLike;
    },
    get audit() {
      return data?.result ?? null;
    },
    get scheduled() {
      return scheduled;
    },
    get published() {
      return published;
    },
    get layout() {
      return preferences.planLayout;
    },
    get dragEnabled() {
      return dragQuery.current;
    },
    get compact() {
      return compactQuery.current;
    },
    get dragging() {
      return dragging;
    },
    lookup: (id) => lookup(id),
    href: (id, term) => href(id, term),
    fceHours: (id) => hours(id),
    isOpen,
    toggleTerm: (term) => {
      openTerms = { ...openTerms, [term]: !isOpen(term) };
    },
    jumpTo,
    addSuggested,
    startDrag: (item) => {
      dragging = item;
    },
    endDrag: () => {
      dragging = null;
    },
  });
</script>

<div class="mx-auto flex w-full max-w-[1600px] flex-col gap-5 px-4 py-5 md:px-6 md:py-6">
  <PlanHeader />
  {#if record.isEmpty}
    <RecordPrompt>
      Your record is empty, so this plan isn't counting past courses or AP, IB or transfer credit toward requirements
      yet.
    </RecordPrompt>
  {/if}
  <RequisiteProblems />

  {#if wideQuery.current}
    <div
      class="grid items-start gap-6 {preferences.planLayout === 'table'
        ? 'grid-cols-[minmax(0,1fr)_clamp(340px,26vw,420px)]'
        : 'grid-cols-[minmax(0,1fr)_clamp(400px,31vw,500px)]'}"
    >
      <!-- The note sits under the semesters so the grid runs to the page's end and the sticky column stays put. -->
      <div class="flex min-w-0 flex-col gap-5">
        <SemesterGrid />
        {@render note()}
      </div>
      <aside
        aria-labelledby="requirements-heading"
        class="scrollbar-thin sticky top-[calc(var(--header-h)+1rem)] flex max-h-[calc(100dvh-var(--header-h)-2.5rem)] flex-col gap-3 overflow-y-auto overscroll-contain pr-1"
      >
        <div class="flex items-center gap-2">
          <h2 id="requirements-heading" class="text-heading-4 text-strong">Requirements</h2>
          <Button size="sm" variant="ghost" icon={Maximize2} class="ml-auto" onclick={() => (expanded = true)}>
            Expand
          </Button>
        </div>
        {#if dragQuery.current && plan.programs.length > 0}
          <p class="-mt-2 text-label-md font-normal text-muted">Drag a course onto a semester, or use Add.</p>
        {/if}
        <RequirementsPanel {data} status={versions.status} error={auditError} drag={dragQuery.current} />
      </aside>
    </div>
  {:else}
    <Tabs
      label="Plan views"
      bind:value={tab}
      tabs={[
        { id: "semesters", label: "Semesters" },
        { id: "requirements", label: "Requirements" },
      ]}
    >
      {#snippet panel(id)}
        {#if id === "requirements"}
          <RequirementsPanel {data} status={versions.status} error={auditError} />
        {:else}
          <SemesterGrid />
        {/if}
      {/snippet}
    </Tabs>
    {@render note()}
  {/if}
</div>

{#snippet note()}
  <p class="text-label-md font-normal text-muted">
    Courses from your <a href={profilePath("taken")} class="text-accent-ink hover:underline">academic record</a> appear in
    their semesters automatically. Unit limits and FCE comparisons are guidance; your college's rules and your advisor have
    the final say.
  </p>
{/snippet}

<Dialog bind:open={expanded} title="Requirements" size="lg">
  <RequirementsPanel {data} status={versions.status} error={auditError} />
</Dialog>
