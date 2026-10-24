<script lang="ts">
  import Button from "$lib/components/ui/Button.svelte";
  import type { RequisiteStatus } from "$lib/components/ui/StatusChip.svelte";
  import type { Unlock } from "$lib/core/catalog";
  import type { CatalogIndex } from "$lib/core/catalog-types";
  import type { Department } from "$lib/core/data";
  import CourseChipList from "./CourseChipList.svelte";
  import RequisiteTreeNode from "./RequisiteTreeNode.svelte";
  import UnlocksList from "./UnlocksList.svelte";

  let {
    id,
    index,
    unlocks,
    href,
    title,
    status,
    coreqStatus,
    offered,
    termName,
    departments,
    headingLevel = 3,
  }: {
    /** The course, as a five-digit id. */
    id: string;
    index: CatalogIndex;
    /** From buildUnlocks(index).get(id). */
    unlocks: readonly Unlock[];
    href: (courseId: string) => string;
    title: (courseId: string) => string | undefined;
    /** How the student stands on each prerequisite; omit for plain chips. */
    status?: (courseId: string) => RequisiteStatus | undefined;
    /** The same for corequisites, where taking them alongside counts. */
    coreqStatus?: (courseId: string) => RequisiteStatus | undefined;
    offered?: (courseId: string) => boolean;
    termName?: string;
    departments?: readonly Department[];
    headingLevel?: 2 | 3;
  } = $props();

  let expandAll = $state(false);
  const entry = $derived(index.courses[id]);
  const ctx = $derived({ index, href, title, status, expandAll });
  const heading = $derived(headingLevel === 2 ? "h2" : "h3");
</script>

<div class="grid min-w-0 gap-5 md:grid-cols-2 md:gap-6">
  <section aria-label="Requires" class="min-w-0 rounded-card border border-pressed bg-surface p-4 sm:p-5">
    <div class="mb-2 flex items-center justify-between gap-3">
      <svelte:element this={heading} class="text-heading-5 text-strong">Requires</svelte:element>
      {#if entry?.prereqs}
        <Button size="sm" variant="ghost" aria-pressed={expandAll} onclick={() => (expandAll = !expandAll)}>
          {expandAll ? "Collapse all" : "Expand all"}
        </Button>
      {/if}
    </div>
    {#if entry?.prereqs}
      <div class="scrollbar-thin -mx-1 overflow-x-auto px-1">
        {#key expandAll}
          <ul>
            <RequisiteTreeNode req={entry.prereqs} {ctx} path={[id]} depth={0} />
          </ul>
        {/key}
      </div>
    {:else}
      <p class="text-body-2 text-muted">
        {entry ? "No prerequisites listed." : "The catalog has no requisite data for this course."}
      </p>
    {/if}
    {#if entry && entry.coreqs.length > 0}
      <div class="mt-4 border-t border-line pt-3">
        <h4 class="text-label-lg text-ink">Corequisites</h4>
        <p class="text-label-md text-muted">
          {entry.coreqs.length > 1 ? "One of these, before or in the same semester" : "Before or in the same semester"}
        </p>
        <CourseChipList
          ids={entry.coreqs}
          status={coreqStatus}
          {href}
          {title}
          joinWord={entry.coreqs.length > 1 ? "or" : undefined}
          class="mt-1.5"
        />
      </div>
    {/if}
    {#if entry && entry.antireqs.length > 0}
      <div class="mt-4 border-t border-line pt-3">
        <h4 class="text-label-lg text-ink">Can't also get credit for</h4>
        <CourseChipList ids={entry.antireqs} {href} {title} class="mt-1.5" />
      </div>
    {/if}
  </section>

  <section aria-label="Unlocks" class="min-w-0 rounded-card border border-pressed bg-surface p-4 sm:p-5">
    <svelte:element this={heading} class="mb-2 text-heading-5 text-strong">Unlocks</svelte:element>
    <!-- Met and missing marks are about this course's requisites; on what it unlocks they'd only read as "not taken". -->
    <UnlocksList {unlocks} {href} {title} {offered} {termName} {departments} />
  </section>
</div>
