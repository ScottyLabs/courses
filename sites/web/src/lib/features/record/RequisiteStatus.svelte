<script lang="ts">
  // One course's requisites as a status line (green met, amber met by planned or same-semester courses, red missing),
  // with details to expand: the prerequisite expression, corequisites, anti-requisites, warnings and buttons to mark
  // requisites waived or undo it. Checked against the academic record, the active degree plan and the semester's
  // active schedule, unless `context` is given. Courseweave's features/record/RequisiteStatus.tsx; for the course and
  // schedule pages.
  import { Check, ChevronDown, Clock, Info, TriangleAlert } from "@lucide/svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import { chipContext, type ChipContext } from "$lib/core/catalog/chip-context";
  import type { Term } from "$lib/core/data";
  import {
    buildRequisiteContext,
    crossListGroups,
    plannedElsewhere,
    scheduleCourseIds,
  } from "$lib/core/record/context";
  import { LEVEL_STYLE, statusFnFor, summarizeCheck } from "$lib/core/record/check-summary";
  import { checkCourse, type RequisiteContext } from "$lib/core/requisites";
  import { coursePath, profilePath } from "$lib/core/routes";
  import { catalog, resource } from "$lib/data";
  import { plans, preferences, record, schedules } from "$lib/state";
  import RequisiteExpression from "./RequisiteExpression.svelte";

  let {
    courseId,
    termId,
    context,
    defaultOpen = false,
    hideWhenClear = false,
    class: className = "",
  }: {
    courseId: string;
    /** The semester the course would be taken in, such as "S27". */
    termId: string;
    /** Check against this instead (the degree planner's own plan). */
    context?: RequisiteContext;
    defaultOpen?: boolean;
    /** Show nothing when there's nothing to report ("No prerequisites"). */
    hideWhenClear?: boolean;
    class?: string;
  } = $props();

  const ICONS = { check: Check, clock: Clock, info: Info, "triangle-alert": TriangleAlert };

  const index = resource(() => catalog.catalogIndex());
  // Cross-listings count for each other; a semester without published course data just goes without them.
  const term = resource(() => catalog.term(termId).catch((): Term | null => null));
  const detailsId = $props.id();
  // Starts as `defaultOpen`; the Details button overrides it.
  let open = $derived(defaultOpen);

  const ctx = $derived.by((): RequisiteContext | null => {
    if (context !== undefined) return context;
    if (index.value === undefined || term.status === "loading") return null;
    const active = schedules.activeId(termId);
    return buildRequisiteContext({
      record: record.data,
      catalog: index.value,
      term: termId,
      sameTerm: scheduleCourseIds(active === null ? null : schedules.get(active)),
      planned: plannedElsewhere({
        termId,
        plan: plans.active,
        schedules: schedules.items,
        activeSchedule: preferences.activeSchedule,
      }),
      equivalentGroups: term.value ? crossListGroups(term.value) : [],
    });
  });
  const entry = $derived(index.value?.courses[courseId]);
  const check = $derived(ctx === null ? null : checkCourse(courseId, entry, ctx));
  const summary = $derived(check === null ? null : summarizeCheck(check, entry));
  const expandable = $derived(
    summary !== null && check !== null && (summary.level !== "none" || check.issues.length > 0),
  );
  const chips = $derived.by((): { prereq: ChipContext; coreq: ChipContext } | null => {
    if (ctx === null) return null;
    const base = {
      termId,
      termName: term.value?.meta.name ?? termId,
      offered: (id: string) => term.value?.courseById.has(id) ?? false,
      title: (id: string) => term.value?.courseById.get(id)?.title ?? index.value?.courses[id]?.title,
      href: (id: string) => coursePath(id, termId),
    };
    return term.value
      ? {
          prereq: chipContext(term.value, index.value ?? null, statusFnFor(ctx, false), base.href),
          coreq: chipContext(term.value, index.value ?? null, statusFnFor(ctx, true), base.href),
        }
      : {
          prereq: { ...base, status: statusFnFor(ctx, false) },
          coreq: { ...base, status: statusFnFor(ctx, true) },
        };
  });
  const override = $derived(record.overrides[courseId]);
  const warnings = $derived(check?.issues.filter((i) => i.severity === "warning") ?? []);
  const prereqMissing = $derived(
    check?.prereqs?.status === "missing" || check?.prereqs?.status === "unknown",
  );
  const coreqMissing = $derived(check?.coreqs?.status === "missing");
</script>

{#snippet waive(key: "prereqs" | "coreqs" | "antireqs", label: string, needed: boolean)}
  {#if check?.waived[key]}
    <Button
      size="sm"
      variant="ghost"
      class="max-md:h-10"
      onclick={() => record.setOverride(courseId, { [key]: false })}
    >
      Undo waiver
    </Button>
  {:else if needed}
    <Button
      size="sm"
      variant="outline"
      class="max-md:h-10"
      onclick={() => record.setOverride(courseId, { [key]: true })}
    >
      {label}
    </Button>
  {/if}
{/snippet}

{#snippet heading(title: string, key: "prereqs" | "coreqs" | "antireqs", label: string, needed: boolean)}
  <div class="flex min-h-10 flex-wrap items-center justify-between gap-x-3 gap-y-1">
    <h4 class="text-label-lg text-strong">{title}</h4>
    {@render waive(key, label, needed)}
  </div>
{/snippet}

{#if index.status === "error"}
  <p class="text-body-2 text-muted {className}">Requisites are unavailable right now: {index.error}</p>
{:else if check === null || summary === null}
  <p class="min-h-6 text-body-2 text-muted {className}" aria-live="polite">Checking requisites&hellip;</p>
{:else if entry === undefined}
  {#if !hideWhenClear}
    <p class="text-body-2 text-muted {className}">No requisite information in the catalog.</p>
  {/if}
{:else if !(hideWhenClear && summary.level === "none")}
  {@const style = LEVEL_STYLE[summary.level]}
  {@const Icon = ICONS[style.icon]}
  <div class="min-w-0 {className}">
    <div class="flex min-h-10 items-start gap-2">
      <Icon aria-hidden="true" class="mt-[3px] size-4 shrink-0 {style.color}" />
      <p class="min-w-0 flex-1 text-body-2 {summary.level === 'problem' ? 'text-danger' : 'text-ink'}">
        {summary.text}
      </p>
      {#if expandable}
        <Button
          size="sm"
          variant="ghost"
          trailingIcon={ChevronDown}
          aria-expanded={open}
          aria-controls={detailsId}
          class="-my-1 shrink-0 max-md:h-10 [&[aria-expanded=true]_svg]:rotate-180"
          onclick={() => (open = !open)}
        >
          {open ? "Hide" : "Details"}
        </Button>
      {/if}
    </div>
    {#if open && chips !== null}
      <div id={detailsId} class="mt-2 flex flex-col gap-3 rounded-field border border-line bg-surface-2 p-3">
        {#if entry.prereqs}
          <section class="flex flex-col gap-1.5">
            {@render heading(
              check.waived.prereqs ? "Prerequisites (waived)" : "Prerequisites",
              "prereqs",
              "Mark as waived",
              prereqMissing,
            )}
            <RequisiteExpression req={entry.prereqs} ctx={chips.prereq} />
          </section>
        {/if}
        {#if entry.coreqs.length > 0}
          <section class="flex flex-col gap-1.5">
            {@render heading(
              check.waived.coreqs
                ? "Corequisite, before or same semester (waived)"
                : "Corequisite, before or same semester",
              "coreqs",
              "Mark as waived",
              coreqMissing,
            )}
            <RequisiteExpression ids={entry.coreqs} joinWord="or" ctx={chips.coreq} />
          </section>
        {/if}
        {#if check.antireqs.length > 0 || check.waived.antireqs}
          <section class="flex flex-col gap-1.5">
            {@render heading("Can't get credit alongside", "antireqs", "Ignore", check.antireqs.length > 0)}
            {#if check.antireqs.length > 0}
              <RequisiteExpression ids={check.antireqs.map((a) => a.courseId)} ctx={chips.prereq} />
            {:else}
              <p class="text-body-2 text-muted">Ignored for this course.</p>
            {/if}
          </section>
        {/if}
        {#if warnings.length > 0}
          <ul class="flex flex-col gap-1">
            {#each warnings as warning (warning.message)}
              <li class="flex items-start gap-2 text-body-2 text-ink">
                <TriangleAlert aria-hidden="true" class="mt-[2px] size-3.5 shrink-0 text-warn" />
                {warning.message}
              </li>
            {/each}
          </ul>
        {/if}
        {#if override?.note}<p class="text-body-2 text-ink">Your note: {override.note}</p>{/if}
        <p class="text-label-md font-normal text-muted">
          Checked against your
          <a href={profilePath("taken")} class="text-accent-ink underline-offset-2 hover:underline">academic record</a>
          and this semester's schedule. Green is done, amber is planned, gray is missing.
        </p>
      </div>
    {/if}
  </div>
{/if}
