<script lang="ts">
  import CourseChip from "$lib/components/ui/CourseChip.svelte";
  import type { RequisiteStatus } from "$lib/components/ui/StatusChip.svelte";
  import { codeOf } from "$lib/core/requisite-format";
  import { coursePath } from "$lib/core/routes";

  let {
    ids,
    status,
    href,
    title,
    termId,
    joinWord,
    class: className = "",
  }: {
    /** Five-digit course ids. */
    ids: readonly string[];
    /** How the student stands on each course; omit for plain chips. */
    status?: (courseId: string) => RequisiteStatus | undefined;
    /** Where each chip links; defaults to the course page. */
    href?: (courseId: string) => string;
    /** Course titles, for tooltips. */
    title?: (courseId: string) => string | undefined;
    /** Semester for the default links. */
    termId?: string | null;
    /** A word between chips, such as "or". */
    joinWord?: string;
    class?: string;
  } = $props();
</script>

<span class="inline-flex min-w-0 flex-wrap items-center gap-1.5 {className}">
  {#each ids as id, index (id)}
    {#if index > 0 && joinWord}<span class="text-body-2 text-ink">{joinWord}</span>{/if}
    <span title={title?.(id)} class="inline-flex">
      <CourseChip code={codeOf(id)} href={href?.(id) ?? coursePath(id, termId)} status={status?.(id)} />
    </span>
  {/each}
</span>
