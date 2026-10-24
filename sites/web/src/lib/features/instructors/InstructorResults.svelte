<script lang="ts">
  import { untrack } from "svelte";
  import { SearchX } from "@lucide/svelte";
  import Title from "$lib/components/shell/Title.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Chip from "$lib/components/ui/Chip.svelte";
  import Skeleton from "$lib/components/ui/Skeleton.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import InstructorCard from "$lib/components/instructor/InstructorCard.svelte";
  import { codeOf } from "$lib/core/requisite-format";
  import { coursePath } from "$lib/core/routes";
  import { teachingSummary } from "./data";
  import InstructorRows from "./InstructorRows.svelte";
  import { useInstructorsScope } from "./scope.svelte";

  // `/instructors`: the results as the Figma's instructor cards, or as Courseweave's rows.
  const scope = useInstructorsScope();
  const PAGE = { cards: 12, list: 80 };

  // More appear as the end of the list comes into view.
  let limit = $state(PAGE.cards);
  $effect(() => {
    void [scope.query, scope.depts, scope.sort, scope.savedOnly, scope.termParam, scope.view];
    limit = untrack(() => PAGE[scope.view]);
  });
  const visible = $derived(scope.results.slice(0, limit));

  function more(element: HTMLElement): () => void {
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting)) limit += PAGE[scope.view];
      },
      { rootMargin: "800px 0px" },
    );
    observer.observe(element);
    return () => observer.disconnect();
  }
</script>

<Title value={scope.allMode ? "Instructors, all semesters" : scope.termMeta ? `Instructors, ${scope.termMeta.name}` : "Instructors"} />

{#if scope.signedIn}
  {#if !scope.ready}
    <div aria-busy="true" class="flex flex-col gap-4">
      <span class="sr-only" aria-live="polite">Loading instructors</span>
      {#each [0, 1, 2] as index (index)}
        <Skeleton class="h-48 w-full rounded-card" />
      {/each}
    </div>
  {:else if scope.results.length === 0}
    <EmptyState
      icon={SearchX}
      title="No instructors match"
      description={scope.total === 0
        ? `No instructors are listed for ${scope.scopeName} yet.`
        : "Try a last name, a course number like 15-122, or clear the filters."}
    >
      {#if scope.filterCount > 0 || scope.query !== ""}
        <Button onclick={() => scope.reset()}>Clear filters</Button>
      {/if}
    </EmptyState>
  {:else if scope.view === "cards"}
    <ul class="flex flex-col gap-6" aria-label="Instructors">
      {#each visible as doc (doc.inst.slug)}
        <li>
          <InstructorCard
            name={doc.inst.name}
            depts={doc.inst.depts}
            href={scope.href(doc.inst.slug)}
            fceCourses={doc.inst.fceCourses}
          >
            {#snippet meta()}
              {#if scope.allMode}
                <p class="mt-2 text-label-md font-normal text-muted">
                  {teachingSummary(doc.inst.terms ?? [])}
                </p>
              {:else}
                <ul class="mt-3 flex flex-wrap gap-1.5" aria-label="Teaching in {scope.scopeName}">
                  {#each doc.inst.courseIds.toSorted().slice(0, 8) as id (id)}
                    <li><Chip size="sm" href={coursePath(id, scope.termId)} label={codeOf(id)} /></li>
                  {/each}
                  {#if doc.inst.courseIds.length > 8}
                    <li><Chip size="sm" tone="neutral" label="+{doc.inst.courseIds.length - 8}" /></li>
                  {/if}
                </ul>
              {/if}
            {/snippet}
          </InstructorCard>
        </li>
      {/each}
    </ul>
  {:else}
    <InstructorRows docs={visible} class="overflow-clip rounded-card border border-pressed bg-surface" />
  {/if}

  {#if scope.ready && visible.length < scope.results.length}
    <div {@attach more} class="mt-6 flex justify-center">
      <Button onclick={() => (limit += PAGE[scope.view])}>Show more instructors</Button>
    </div>
  {/if}
{/if}
