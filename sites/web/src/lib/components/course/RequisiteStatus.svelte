<script lang="ts">
  // One course's requisites as a status line (green met, amber met by planned or same-semester courses, red missing),
  // with details to expand: the prerequisite expression, corequisites, anti-requisites, warnings and buttons to mark
  // requisites waived or undo it. Checked against the academic record, the active degree plan and the semester's
  // active schedule. Courseweave's features/record/RequisiteStatus.tsx; for the course page and its detail panel.
  import { ChevronDown, Info, TriangleAlert } from "@lucide/svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import { summarizeCheck } from "$lib/core/record/check-summary";
  import { codeOf } from "$lib/core/requisite-format";
  import { checkCourse, describeRequisite } from "$lib/core/requisites";
  import { profilePath } from "$lib/core/routes";
  import { record } from "$lib/state";
  import CourseChipList from "./CourseChipList.svelte";
  import { catalogIndex } from "./data.svelte";
  import { getCourseHref } from "./links";
  import { termRequisites } from "./requisite-context.svelte";
  import RequisiteExpression from "./RequisiteExpression.svelte";
  import RequisiteLevelIcon from "./RequisiteLevelIcon.svelte";
  import { textAction } from "./text-action";

  let {
    courseId,
    termId,
    class: className = "",
  }: {
    courseId: string;
    /** The semester the course would be taken in. */
    termId: string;
    class?: string;
  } = $props();

  type Key = "prereqs" | "coreqs" | "antireqs";

  const detailsId = $props.id();
  let open = $state(false);
  const index = $derived(catalogIndex());
  const requisites = $derived(termRequisites(termId));
  const ctx = $derived(requisites.ctx);
  const entry = $derived(index.value?.courses[courseId]);
  const check = $derived(ctx ? checkCourse(courseId, entry, ctx) : null);
  const summary = $derived(check ? summarizeCheck(check, entry) : null);
  const expandable = $derived(
    summary !== null && check !== null && (summary.level !== "none" || check.issues.length > 0),
  );
  const override = $derived(record.overrides[courseId]);
  const prereqMissing = $derived(check?.prereqs?.status === "missing" || check?.prereqs?.status === "unknown");
  const coreqMissing = $derived(check?.coreqs?.status === "missing");
  const warnings = $derived(check?.issues.filter((i) => i.severity === "warning") ?? []);
  const titleOf = (id: string): string | undefined => index.value?.courses[id]?.title;
  const courseHref = getCourseHref();
  const hrefOf = (id: string): string => courseHref(id, termId);
  // Nothing taken, planned or scheduled yet: every check would read "missing", which says nothing. With an empty
  // record, a missing prerequisite most likely means "not entered yet", so it asks for the record rather than
  // reporting an error, even once schedules or a plan give it courses to check against.
  const unchecked = $derived(
    ctx !== null &&
      (ctx.credits.size === 0 ||
        (record.isEmpty && summary?.level === "problem" && check !== null && check.antireqs.length === 0)),
  );
  const plainText = $derived.by(() => {
    if (!entry) return "";
    const parts = [entry.prereqs ? `Requires ${describeRequisite(entry.prereqs)}` : null];
    const coreqs = entry.coreqs.filter((id) => id !== courseId);
    if (coreqs.length > 0)
      parts.push(`${entry.prereqs ? "and" : "Requires"} ${coreqs.length > 1 ? "one of " : ""}${coreqs.map((id) => codeOf(id)).join(", ")} before or alongside`);
    return parts.filter((p) => p !== null).join(" ");
  });

  function waive(key: Key, on: boolean): void {
    record.setOverride(courseId, { [key]: on });
  }
</script>

{#snippet waiveButton(key: Key, label: string, needed: boolean)}
  {#if check?.waived[key]}
    <Button size="sm" variant="ghost" class="max-md:h-10" onclick={() => waive(key, false)}>Undo waiver</Button>
  {:else if needed}
    <Button size="sm" variant="secondary" class="max-md:h-10" onclick={() => waive(key, true)}>{label}</Button>
  {/if}
{/snippet}

<!--
  The status text's last word with Details right after it, kept on one line so Details never starts a line on its own;
  its negative margins keep the line 20px tall.
-->
{#snippet withToggle(text: string)}
  {@const cut = text.lastIndexOf(" ") + 1}
  {#if expandable}{text.slice(0, cut)}<span class="whitespace-nowrap"
      >{text.slice(cut)}&nbsp;<button
        type="button"
        aria-expanded={open}
        aria-controls={detailsId}
        class="{textAction} -my-1.5 ml-0.5 align-baseline max-md:-my-2.5"
        onclick={() => (open = !open)}
      >
        {open ? "Hide details" : "Details"}<ChevronDown
          aria-hidden="true"
          class="size-4 transition-transform {open ? 'rotate-180' : ''}"
        />
      </button></span
    >{:else}{text}{/if}
{/snippet}

<div class="min-w-0 {className}">
  <!--
    The status line, announced as it changes; Details follows its last word, so it stays with the status however the
    line wraps. The details opened under it aren't announced.
  -->
  <div aria-live="polite">
    {#if index.status === "error"}
      <p class="text-body-2 text-muted">Requisites didn't load: {index.error}</p>
    {:else if !summary || !check}
      <p class="text-body-2 text-muted">Checking requisites&hellip;</p>
    {:else if index.status === "ready" && !entry}
      <p class="text-body-2 text-muted">No requisite information in the catalog.</p>
    {:else}
      <div class="flex items-start gap-2">
        {#if unchecked && summary.level !== "none"}
          <Info aria-hidden="true" class="mt-[3px] size-4 shrink-0 text-muted" />
          <p class="min-w-0 flex-1 text-body-2 text-ink">
            {plainText}.
            <a href={profilePath("taken")} class="text-accent-ink underline underline-offset-2"
              >Add the courses you've taken</a
            >{@render withToggle(" to check them.")}
          </p>
        {:else}
          <RequisiteLevelIcon level={summary.level} class="mt-[3px] size-4" />
          <p class="min-w-0 flex-1 text-body-2 {summary.level === 'problem' ? 'text-danger' : 'text-ink'}">
            {@render withToggle(summary.text)}
          </p>
        {/if}
      </div>
    {/if}
  </div>
  {#if open && entry && ctx && check}
    <div id={detailsId} class="mt-3 flex flex-col gap-3 rounded-card border border-line bg-surface-2 p-3 text-left">
      {#if entry.prereqs}
        <section class="flex flex-col gap-1.5">
          <div class="flex min-h-8 flex-wrap items-center justify-between gap-x-3 gap-y-1">
            <h3 class="text-label-lg text-ink">{check.waived.prereqs ? "Prerequisites (waived)" : "Prerequisites"}</h3>
            {@render waiveButton("prereqs", "Mark as waived", prereqMissing)}
          </div>
          <RequisiteExpression
            req={entry.prereqs}
            status={unchecked ? undefined : (requisites.prereqStatus ?? undefined)}
            title={titleOf}
            href={hrefOf}
          />
        </section>
      {/if}
      {#if entry.coreqs.length > 0}
        <section class="flex flex-col gap-1.5">
          <div class="flex min-h-8 flex-wrap items-center justify-between gap-x-3 gap-y-1">
            <h3 class="text-label-lg text-ink">
              Corequisite, before or same semester{check.waived.coreqs ? " (waived)" : ""}
            </h3>
            {@render waiveButton("coreqs", "Mark as waived", coreqMissing)}
          </div>
          <CourseChipList
            ids={entry.coreqs}
            status={unchecked ? undefined : (requisites.coreqStatus ?? undefined)}
            title={titleOf}
            href={hrefOf}
            joinWord="or"
          />
        </section>
      {/if}
      {#if check.antireqs.length > 0 || check.waived.antireqs}
        <section class="flex flex-col gap-1.5">
          <div class="flex min-h-8 flex-wrap items-center justify-between gap-x-3 gap-y-1">
            <h3 class="text-label-lg text-ink">Can't get credit alongside</h3>
            {@render waiveButton("antireqs", "Ignore", check.antireqs.length > 0)}
          </div>
          {#if check.antireqs.length > 0}
            <CourseChipList ids={check.antireqs.map((a) => a.courseId)} title={titleOf} href={hrefOf} />
          {:else}
            <p class="text-body-2 text-muted">Ignored for this course.</p>
          {/if}
        </section>
      {/if}
      {#if warnings.length > 0}
        <ul class="flex flex-col gap-1">
          {#each warnings as w (w.message)}
            <li class="flex items-start gap-2 text-body-2 text-ink">
              <TriangleAlert aria-hidden="true" class="mt-[3px] size-3.5 shrink-0 text-warn" />{w.message}
            </li>
          {/each}
        </ul>
      {/if}
      {#if override?.note}<p class="text-body-2 text-ink">Your note: {override.note}</p>{/if}
      <p class="text-label-md text-muted">
        Checked against your
        <a href={profilePath("taken")} class="text-accent-ink underline underline-offset-2">courses taken</a>, degree
        plan and this semester's schedule. Green is done and yellow is planned.
      </p>
    </div>
  {/if}
</div>
