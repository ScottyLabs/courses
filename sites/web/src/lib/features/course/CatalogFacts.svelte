<script lang="ts">
  import { catalogIndex, offerings } from "$lib/components/course/data.svelte";
  import { departmentName } from "$lib/core/departments";
  import { termLabel } from "$lib/core/requisites";
  import { coursesPath } from "$lib/core/routes";
  import type { TermIndexEntry } from "$lib/core/types";
  import { getCourseHref } from "./links";

  let {
    courseId,
    terms,
    layout = "row",
  }: {
    courseId: string;
    /** Every built semester, newest first, to say which span "not offered" covers. */
    terms: readonly TermIndexEntry[];
    /** row: side by side (full page); list: label and value rows (detail panel). */
    layout?: "row" | "list";
  } = $props();

  // What doesn't depend on a semester (the all-semesters view): from the catalog and the offerings document.
  const courseHref = getCourseHref();
  // Loads are requested in their own deriveds: a reaction doesn't track state it created itself.
  const index = $derived(catalogIndex());
  const all = $derived(offerings());
  const entry = $derived(index.value?.courses[courseId]);
  const offering = $derived(all.value?.byId.get(courseId));
  const offered = $derived(offering?.terms ?? []);
  const level = $derived(offering?.level ?? (Number(courseId[2]) >= 6 ? "G" : "U"));
  const span = $derived.by(() => {
    const oldest = terms.at(-1);
    const newest = terms[0];
    return oldest && newest ? `${oldest.name} to ${newest.name}` : null;
  });
  const units = $derived.by(() => {
    if (!entry) return "Unknown";
    if (entry.unitsMax !== undefined) return `Variable, ${entry.units ?? 0}\u2013${entry.unitsMax}`;
    return entry.units === null ? "Unknown" : `${entry.units} ${entry.units === 1 ? "unit" : "units"}`;
  });
  const rows = $derived([
    { label: "Units", text: units },
    ...(offering?.dept ? [{ label: "Department", text: departmentName(offering.dept), dept: offering.dept }] : []),
    { label: "Level", text: level === "G" ? "Graduate" : "Undergraduate" },
  ]);
</script>

<dl
  class={layout === "row"
    ? "flex flex-wrap gap-x-10 gap-y-4 text-body-2 text-ink"
    : "grid grid-cols-[7rem_minmax(0,1fr)] gap-x-3 gap-y-1.5 text-body-2 text-ink"}
>
  {#each rows as row (row.label)}
    <div class={layout === "row" ? "min-w-0" : "contents"}>
      <dt class={layout === "row" ? "text-label-md text-muted" : "text-muted"}>{row.label}</dt>
      <dd class={layout === "row" ? "mt-0.5" : ""}>
        {#if row.dept}
          <a
            href="{coursesPath('all')}&dept={encodeURIComponent(row.dept)}"
            class="underline decoration-line-strong underline-offset-2 hover:text-accent-ink">{row.text}</a
          >
          <span class="text-muted">({row.dept})</span>
        {:else}
          {row.text}
        {/if}
      </dd>
    </div>
  {/each}
  <div class={layout === "row" ? "max-w-[30rem] min-w-0" : "contents"}>
    <dt class={layout === "row" ? "text-label-md text-muted" : "text-muted"}>Offered</dt>
    <dd class={layout === "row" ? "mt-0.5" : ""}>
      {#if offered.length > 0}
        <span class="flex flex-wrap gap-x-3">
          {#each offered as t (t)}
            <a
              href={courseHref(courseId, t)}
              class="underline decoration-line-strong underline-offset-2 hover:text-accent-ink">{termLabel(t)}</a
            >
          {/each}
        </span>
      {:else}
        <span class="text-muted">Not in any semester this site has{span ? ` (${span})` : ""}</span>
      {/if}
    </dd>
  </div>
</dl>
