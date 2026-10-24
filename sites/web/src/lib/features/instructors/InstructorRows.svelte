<script lang="ts">
  import InstructorRow from "$lib/components/instructor/InstructorRow.svelte";
  import type { Instructor } from "$lib/core/data";
  import { teachingSummary } from "./data";
  import type { ListedDoc } from "./list";
  import { useInstructorsScope } from "./scope.svelte";

  // Courseweave's instructor rows: the list view, and the column beside an open instructor on wide screens.
  let {
    docs,
    active = -1,
    stickyTop = "top-(--header-h)",
    class: className = "",
  }: {
    /** The rows to show (the visible part of the results). */
    docs: readonly ListedDoc[];
    /** The row the arrow keys are on, from the search field. */
    active?: number;
    /** Where the letter headings stick. */
    stickyTop?: string;
    class?: string;
  } = $props();

  const scope = useInstructorsScope();

  function countText(inst: Instructor): string {
    const sections = scope.sort === "sections";
    const n = sections ? inst.sectionCount : inst.courseIds.length;
    return `${n} ${sections ? "section" : "course"}${n === 1 ? "" : "s"}`;
  }
</script>

<ul id="instructor-results" role="listbox" aria-label="Instructors" class="@container {className}">
  {#each docs as doc, index (doc.inst.slug)}
    {#if scope.grouped && (index === 0 || docs[index - 1]?.letter !== doc.letter)}
      <li
        role="presentation"
        aria-hidden="true"
        class="sticky {stickyTop} z-[1] border-b border-line bg-surface-2 px-4 py-1 text-label-md text-muted sm:px-5"
      >
        {doc.letter}
      </li>
    {/if}
    <li role="presentation" class="border-b border-line last:border-b-0">
      <InstructorRow
        id="instructor-row-{index}"
        name={doc.inst.name}
        href={scope.href(doc.inst.slug)}
        depts={doc.inst.depts}
        detail={doc.inst.terms === null ? undefined : teachingSummary(doc.inst.terms)}
        count={countText(doc.inst)}
        rating={doc.inst.rating}
        selected={doc.inst.slug === scope.slug}
        active={index === active}
      />
    </li>
  {/each}
</ul>
