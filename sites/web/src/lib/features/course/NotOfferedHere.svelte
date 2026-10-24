<script lang="ts">
  import { ArrowRight, GitFork } from "@lucide/svelte";
  import { catalogIndex, offerings } from "$lib/components/course/data.svelte";
  import { getCourseHref } from "$lib/components/course/links";
  import Button from "$lib/components/ui/Button.svelte";
  import Skeleton from "$lib/components/ui/Skeleton.svelte";
  import { nearestTerm } from "$lib/core/offerings";
  import { codeOf } from "$lib/core/requisite-format";
  import { termLabel } from "$lib/core/requisites";
  import { coursePath, coursesPath } from "$lib/core/routes";
  import { catalogTitle } from "$lib/core/explorer/unoffered";
  import type { TermIndexEntry } from "$lib/core/types";
  import { requisitesPath } from "./paths";

  let {
    courseId,
    termId,
    terms,
    backHref,
    allHref,
    compact = false,
  }: {
    courseId: string;
    /** The semester asked for, which doesn't offer it. */
    termId: string;
    terms: readonly TermIndexEntry[];
    /** "Back to all courses"; defaults to the semester's course list. */
    backHref?: string;
    /** The course in the all-semesters view; defaults to its page there. */
    allHref?: string;
    /** In the detail panel: smaller type, headings one level down. */
    compact?: boolean;
  } = $props();

  const courseHref = getCourseHref();

  const all = $derived(offerings());
  const index = $derived(catalogIndex());
  const offering = $derived(all.value?.byId.get(courseId));
  const title = $derived(offering?.title ?? catalogTitle(index.value?.courses[courseId]?.title));
  const next = $derived(offering ? nearestTerm(offering.terms, termId) : undefined);
  const termName = $derived(terms.find((t) => t.id === termId)?.name ?? termLabel(termId));
  const code = $derived(codeOf(courseId));
</script>

<div class="grid place-items-center text-center {compact ? 'px-4 py-8' : 'py-10'}">
  {#if all.status === "loading" || index.status === "loading"}
    <div class="flex w-full max-w-sm flex-col items-center gap-3" aria-busy="true">
      <Skeleton class="h-10 w-32" />
      <Skeleton variant="text" lines={2} class="w-full" />
    </div>
  {:else}
    <div class="max-w-md">
      <p class="text-[40px] leading-none font-semibold tracking-tight text-muted">{code}</p>
      <svelte:element this={compact ? "h2" : "h1"} class="mt-2 mb-3 text-strong {compact ? 'text-heading-4' : 'text-heading-3'}">
        {title ?? `No course ${code}`}
      </svelte:element>
      <p class="mb-5 text-body-1 text-ink">
        {#if offering}
          Not offered in {termName}. Offered in
          {#each offering.terms as t, i (t)}{i > 0 ? ", " : ""}<a
              href={courseHref(courseId, t)}
              class="whitespace-nowrap text-accent-ink underline underline-offset-2">{termLabel(t)}</a
            >{/each}.
        {:else if title}
          Not offered in {termName} or any other semester this site has.
        {:else}
          It isn't offered in {termName}, or the number is mistyped.
        {/if}
      </p>
      <div class="flex flex-wrap justify-center gap-2">
        {#if next}
          <Button variant="primary" trailingIcon={ArrowRight} href={courseHref(courseId, next)}>
            View in {termLabel(next)}
          </Button>
        {/if}
        {#if title}
          <Button href={allHref ?? coursePath(courseId, "all")}>Course details</Button>
          <Button icon={GitFork} href={requisitesPath(courseId, termId)}>Prerequisites</Button>
        {/if}
      </div>
      <a
        href={backHref ?? coursesPath(termId)}
        class="mt-5 inline-flex min-h-10 items-center text-label-lg text-accent-ink underline-offset-2 hover:underline"
      >
        Back to all courses
      </a>
    </div>
  {/if}
</div>
