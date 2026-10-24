<script lang="ts">
  // AP, IB and transfer credit and placements from the record, above the semesters.
  import { profilePath } from "$lib/core/routes";
  import { planPage } from "./context";

  const page = planPage();
  const incoming = $derived(page.derived.incoming);
  const KIND = { exam: "Exam", transfer: "Transfer", waiver: "Placement", taken: "Course" } as const;

  function unitsOf(e: (typeof incoming)[number]): number {
    if (e.source === "waiver") return 0;
    return e.units ?? (e.courseId === null ? 0 : (page.lookup(e.courseId).units ?? 0));
  }
  const units = $derived(incoming.reduce((n, e) => n + unitsOf(e), 0));
</script>

{#if incoming.length > 0}
  <section aria-labelledby="incoming-credit" class="rounded-card border border-pressed bg-surface">
    <header class="flex flex-wrap items-baseline gap-x-3 gap-y-1 border-b border-line px-4 py-2.5">
      <h2 id="incoming-credit" class="text-heading-6 text-strong">Incoming credit</h2>
      <span class="tabular text-label-md font-normal text-muted">{units} units</span>
      <a href={profilePath("taken")} class="ml-auto text-label-md text-accent-ink hover:underline">Edit in your record</a>
    </header>
    <ul class="grid grid-cols-1 gap-x-4 px-2 py-1.5 @2xl:grid-cols-2 @5xl:grid-cols-3">
      {#each incoming as e (e.id)}
        {@const info = e.courseId === null ? null : page.lookup(e.courseId)}
        <li class="flex min-h-10 items-center gap-1.5 px-2 md:min-h-8">
          <span class="tabular shrink-0 text-label-lg text-ink">{info?.code ?? "Units"}</span>
          <span class="min-w-0 flex-1 truncate text-body-2 text-muted">
            {e.label ?? info?.title ?? e.institution ?? "General credit"}
          </span>
          <span class="shrink-0 text-label-md font-normal text-muted">{KIND[e.source]}</span>
          <span class="tabular w-9 shrink-0 text-right text-label-md font-normal text-muted">{unitsOf(e)}u</span>
        </li>
      {/each}
    </ul>
  </section>
{/if}
