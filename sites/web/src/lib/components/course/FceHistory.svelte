<script lang="ts">
  import { ArrowDown, ArrowUp } from "@lucide/svelte";
  import SegmentedControl from "$lib/components/ui/SegmentedControl.svelte";
  import { instructorSlug } from "$lib/core/data";
  import {
    expectedHours,
    FCE_HOURS_HELP,
    formatHours,
    formatRating,
    rowsExpectedHours,
    sameInstructor,
    termName,
    termOrder,
    WORKLOAD_TONE_LABEL,
    weightedMean,
    workloadTone,
    type FceRow,
  } from "$lib/core/fce";
  import { displayFceName, fceNameKeys } from "$lib/core/fce-names";
  import { ALL_TERMS, instructorPath } from "$lib/core/routes";
  import { TONE_TEXT } from "./WorkloadMeter.svelte";

  type Mode = "instructor" | "term";
  type SortDir = "asc" | "desc";

  let {
    rows,
    current = [],
    units,
    fixedMode,
    currentLabel = "Teaching this semester",
  }: {
    /** Every FCE row for the course. */
    rows: readonly FceRow[];
    /** Instructors teaching it in the semester shown, as the schedule names them; marked and listed first. */
    current?: readonly string[];
    units: number | null;
    /** Show one grouping without the switch (the wide layout shows both side by side). */
    fixedMode?: Mode;
    currentLabel?: string;
  } = $props();

  interface InstructorRow {
    key: string;
    name: string;
    terms: number;
    last: string;
    responses: number;
    teaching: number | null;
    courseRate: number | null;
    hours: number | null;
    /** Hours their sections' units suggest (2 x units in mini sections), response-weighted. */
    expected: number | null;
    miniShare: number;
    current: boolean;
  }

  const PREVIEW_ROWS = 8;
  const weight = (r: FceRow): number => r.responses;

  let chosenMode = $state<Mode>("instructor");
  let sort = $state<{ key: string; dir: SortDir }>({ key: "last", dir: "desc" });
  let expanded = $state(false);
  const mode = $derived(fixedMode ?? chosenMode);
  const dir = $derived(sort.dir === "asc" ? 1 : -1);

  const instructors = $derived.by((): InstructorRow[] => {
    const groups = new Map<string, FceRow[]>();
    for (const r of rows) {
      if (!r.instructor) continue;
      const key = fceNameKeys(r.instructor).full;
      groups.set(key, [...(groups.get(key) ?? []), r]);
    }
    return [...groups.entries()].map(([key, list]) => {
      const first = list[0];
      const raw = first?.instructor ?? "";
      const answered = list.filter((r) => r.responses > 0);
      const terms = [...new Set(list.map((r) => r.term))].toSorted((a, b) => termOrder(b) - termOrder(a));
      const currentName = current.find((n) => sameInstructor(raw, n));
      return {
        key,
        name: currentName ?? displayFceName(raw),
        terms: terms.length,
        last: terms[0] ?? "",
        responses: answered.reduce((n, r) => n + r.responses, 0),
        teaching: weightedMean(answered, (r) => r.teachingRate, weight),
        courseRate: weightedMean(answered, (r) => r.courseRate, weight),
        hours: weightedMean(answered, (r) => r.hours, weight),
        expected: rowsExpectedHours(answered, units),
        miniShare: weightedMean(answered, (r) => (r.mini === null ? 0 : 1), weight) ?? 0,
        current: currentName !== undefined,
      };
    });
  });

  function cmp(a: number | string | null, b: number | string | null): number {
    if (a === null || b === null) {
      if (a === null) return b === null ? 0 : 1;
      return -1;
    }
    if (typeof a === "string" || typeof b === "string") return String(a).localeCompare(String(b));
    return a - b;
  }

  const sortedInstructors = $derived.by(() => {
    const value = (r: InstructorRow): number | string | null => {
      switch (sort.key) {
        case "name":
          return r.name;
        case "terms":
          return r.terms;
        case "teaching":
          return r.teaching;
        case "course":
          return r.courseRate;
        case "hours":
          return r.hours;
        default:
          return termOrder(r.last);
      }
    };
    // Current instructors first when sorted by recency, so section choices are easy to compare.
    return instructors.toSorted(
      (a, b) =>
        (sort.key === "last" ? Number(b.current) - Number(a.current) : 0) ||
        dir * cmp(value(a), value(b)) ||
        a.name.localeCompare(b.name),
    );
  });

  const sortedRows = $derived.by(() => {
    const value = (r: FceRow): number | string | null => {
      switch (sort.key) {
        case "name":
          return displayFceName(r.instructor);
        case "teaching":
          return r.teachingRate;
        case "course":
          return r.courseRate;
        case "hours":
          return r.hours;
        case "responses":
          return r.responses;
        default:
          return termOrder(r.term);
      }
    };
    return rows.toSorted(
      (a, b) =>
        dir * cmp(value(a), value(b)) ||
        termOrder(b.term) - termOrder(a.term) ||
        a.section.localeCompare(b.section),
    );
  });

  const total = $derived(mode === "instructor" ? sortedInstructors.length : sortedRows.length);
  const shown = $derived(expanded ? total : Math.min(total, PREVIEW_ROWS));

  function sortBy(key: string, defaultDir: SortDir): void {
    sort = sort.key === key ? { key, dir: sort.dir === "asc" ? "desc" : "asc" } : { key, dir: defaultDir };
  }

  function hoursTitle(hours: number | null, expected: number | null, mini: boolean): string | undefined {
    if (hours === null) return undefined;
    const tone = expected ? workloadTone(hours / expected) : null;
    const against =
      expected === null ? "" : ` against about ${formatHours(expected)} its units suggest${mini ? " as a mini" : ""}`;
    return `${formatHours(hours)} hours a week${against}${tone ? `: ${WORKLOAD_TONE_LABEL[tone].toLowerCase()}` : ""}`;
  }
  function hoursClass(hours: number | null, expected: number | null): string {
    const tone = hours !== null && expected ? workloadTone(hours / expected) : null;
    return tone && tone !== "even" ? TONE_TEXT[tone] : "text-ink";
  }
  const slugOf = (raw: string, name: string): string => instructorSlug(raw ? displayFceName(raw) : name);
</script>

{#snippet th(key: string, label: string, align: "left" | "right", defaultDir: SortDir, title: string | undefined, hide: boolean)}
  <th
    scope="col"
    aria-sort={sort.key === key ? (sort.dir === "asc" ? "ascending" : "descending") : "none"}
    class="px-2 py-1.5 font-semibold {align === 'right' ? 'text-right' : 'text-left'} {hide
      ? 'hidden sm:table-cell'
      : ''}"
  >
    <button
      type="button"
      onclick={() => sortBy(key, defaultDir)}
      {title}
      class="inline-flex min-h-8 items-center gap-0.5 rounded-tag hover:text-strong"
    >
      {label}
      {#if sort.key === key}
        {#if sort.dir === "asc"}<ArrowUp aria-hidden="true" class="size-3" />{:else}<ArrowDown
            aria-hidden="true"
            class="size-3"
          />{/if}
      {/if}
    </button>
  </th>
{/snippet}

<div class="mt-6 min-w-0">
  <div class="mb-2 flex flex-wrap items-center justify-between gap-3">
    <h3 class="text-heading-6 text-strong">{fixedMode === "term" ? "History by semester" : fixedMode === "instructor" ? "History by instructor" : "History"}</h3>
    {#if !fixedMode}
      <SegmentedControl
        size="sm"
        label="Group history"
        options={[
          { value: "instructor", label: "By instructor" },
          { value: "term", label: "By semester" },
        ]}
        bind:value={() => chosenMode, (next) => (chosenMode = next === "term" ? "term" : "instructor")}
        onchange={() => (expanded = false)}
      />
    {/if}
  </div>

  <div class="data-table-scroll rounded-card bg-surface-2 px-2 py-2 sm:px-3">
    <table class="w-full text-left text-body-2 text-ink">
      <caption class="sr-only">Course evaluations {mode === "instructor" ? "by instructor" : "by semester"}</caption>
      <thead class="text-ink">
        {#if mode === "instructor"}
          <tr>
            {@render th("name", "Instructor", "left", "asc", undefined, false)}
            {@render th("terms", "Semesters", "right", "desc", undefined, true)}
            {@render th("last", "Last", "right", "desc", undefined, false)}
            {@render th("teaching", "Teaching", "right", "desc", "Overall teaching rating, 1 to 5", false)}
            {@render th("course", "Course", "right", "desc", "Overall course rating, 1 to 5", false)}
            {@render th("hours", "Hrs/wk", "right", "asc", FCE_HOURS_HELP, false)}
          </tr>
        {:else}
          <tr>
            {@render th("last", "Semester", "left", "desc", undefined, false)}
            <th scope="col" class="hidden px-2 py-1.5 font-semibold sm:table-cell">Section</th>
            {@render th("name", "Instructor", "left", "asc", undefined, false)}
            {@render th("responses", "Responses", "right", "desc", "Responses out of enrolled students", true)}
            {@render th("teaching", "Teaching", "right", "desc", "Overall teaching rating, 1 to 5", false)}
            {@render th("course", "Course", "right", "desc", "Overall course rating, 1 to 5", false)}
            {@render th("hours", "Hrs/wk", "right", "asc", FCE_HOURS_HELP, false)}
          </tr>
        {/if}
      </thead>
      <tbody class="tabular font-medium">
        {#if mode === "instructor"}
          {#each sortedInstructors.slice(0, shown) as r (r.key)}
            <tr>
              <td class="max-w-[14rem] px-2 py-1.5">
                <a
                  href={instructorPath(slugOf("", r.name), ALL_TERMS)}
                  class="block truncate underline decoration-line-strong underline-offset-2 hover:text-accent-ink {r.current
                    ? 'font-semibold text-strong'
                    : ''}"
                  title={r.current ? `${r.name}: ${currentLabel.toLowerCase()}` : r.name}>{r.name}</a
                >
                {#if r.current}<span class="text-label-md text-accent-ink">{currentLabel}</span>{/if}
              </td>
              <td class="hidden px-2 py-1.5 text-right sm:table-cell">{r.terms}</td>
              <td class="px-2 py-1.5 text-right" title={r.last ? termName(r.last) : undefined}>{r.last}</td>
              <td class="px-2 py-1.5 text-right">{formatRating(r.teaching)}</td>
              <td class="px-2 py-1.5 text-right">{formatRating(r.courseRate)}</td>
              <td
                class="px-2 py-1.5 text-right {hoursClass(r.hours, r.expected)}"
                title={hoursTitle(r.hours, r.expected, r.miniShare >= 0.5)}>{formatHours(r.hours)}</td
              >
            </tr>
          {/each}
        {:else}
          {#each sortedRows.slice(0, shown) as r, i (`${r.term}-${r.section}-${r.instructor}-${i}`)}
            {@const expected = expectedHours(units, r.mini !== null)}
            <tr>
              <td class="px-2 py-1.5 whitespace-nowrap" title={termName(r.term)}>{r.term}</td>
              <td class="hidden px-2 py-1.5 sm:table-cell">
                {r.section}{#if r.mini !== null}<span class="ml-1 text-label-md text-muted">mini</span>{/if}
              </td>
              <td class="max-w-[12rem] truncate px-2 py-1.5">
                {#if r.instructor}
                  <a
                    href={instructorPath(slugOf(r.instructor, ""), ALL_TERMS)}
                    class="underline decoration-line-strong underline-offset-2 hover:text-accent-ink"
                    >{displayFceName(r.instructor)}</a
                  >
                {:else}-{/if}
              </td>
              <td class="hidden px-2 py-1.5 text-right text-muted sm:table-cell">{r.responses}/{r.enrolled}</td>
              <td class="px-2 py-1.5 text-right">{r.responses > 0 ? formatRating(r.teachingRate) : "-"}</td>
              <td class="px-2 py-1.5 text-right">{r.responses > 0 ? formatRating(r.courseRate) : "-"}</td>
              <td
                class="px-2 py-1.5 text-right {r.responses > 0 ? hoursClass(r.hours, expected) : ''}"
                title={r.responses > 0 ? hoursTitle(r.hours, expected, r.mini !== null) : undefined}
                >{r.responses > 0 ? formatHours(r.hours) : "-"}</td
              >
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </div>
  {#if total > PREVIEW_ROWS}
    <button
      type="button"
      onclick={() => (expanded = !expanded)}
      class="mt-2 inline-flex min-h-10 items-center text-label-lg text-accent-ink underline-offset-2 hover:underline sm:min-h-8"
    >
      {expanded ? "Show fewer" : `Show all ${total} ${mode === "instructor" ? "instructors" : "rows"}`}
    </button>
  {/if}
</div>
