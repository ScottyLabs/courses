<script lang="ts">
  import CourseChip from "$lib/components/ui/CourseChip.svelte";
  import Chip from "$lib/components/ui/Chip.svelte";
  import type { Unlock } from "$lib/core/catalog";
  import { departmentName } from "$lib/core/departments";
  import type { Department } from "$lib/core/data";
  import { codeOf } from "$lib/core/requisite-format";

  let {
    unlocks,
    href,
    title,
    offered,
    termName,
    departments = [],
  }: {
    /** From buildUnlocks(index).get(courseId). */
    unlocks: readonly Unlock[];
    href: (courseId: string) => string;
    title: (courseId: string) => string | undefined;
    /** Whether the semester offers a course; omit in the all-semesters view to hide that filter. */
    offered?: (courseId: string) => boolean;
    /** The semester's name, for the "Offered in" filter. */
    termName?: string;
    /** The semester's departments, to name course-number prefixes. */
    departments?: readonly Department[];
  } = $props();

  const GROUP_PREVIEW = 8;

  let offeredOnly = $state(false);
  let requiredOnly = $state(false);
  let expanded = $state<string[]>([]);

  const prefixName = $derived.by(() => {
    const names = new Map<string, string>();
    for (const department of departments)
      for (const prefix of department.prefixes) if (!names.has(prefix)) names.set(prefix, department.name);
    return (prefix: string): string => names.get(prefix) ?? departmentName(prefix);
  });
  const shown = $derived(
    unlocks.filter(
      (u) => (!offeredOnly || offered === undefined || offered(u.id)) && (!requiredOnly || u.required),
    ),
  );
  const groups = $derived.by(() => {
    const map = new Map<string, Unlock[]>();
    for (const u of shown) {
      const prefix = u.id.slice(0, 2);
      map.set(prefix, [...(map.get(prefix) ?? []), u]);
    }
    return [...map].toSorted((a, b) => b[1].length - a[1].length || a[0].localeCompare(b[0]));
  });
  const requiredCount = $derived(unlocks.filter((u) => u.required).length);

  function kindOf(u: Unlock): { text: string; help: string } {
    if (u.via === "coreq") return { text: "Corequisite", help: "Take before or alongside" };
    if (u.required) return { text: "Required", help: "Every prerequisite path needs this course" };
    return { text: "One option", help: "One of several ways to meet the prerequisites" };
  }
</script>

{#if unlocks.length === 0}
  <p class="text-body-2 text-muted">No course lists this as a prerequisite or corequisite.</p>
{:else}
  <div class="flex min-w-0 flex-col gap-3">
    <p class="text-body-2 text-ink">
      {unlocks.length === 1 ? "1 course lists" : `${unlocks.length} courses list`} it as a requisite;
      {requiredCount === unlocks.length ? "all" : requiredCount} of them can't be taken without it.
    </p>
    <div class="flex flex-wrap gap-2">
      <Chip
        shape="pill"
        size="md"
        pressed={requiredOnly}
        title="Only courses whose every prerequisite path needs this course"
        onclick={() => (requiredOnly = !requiredOnly)}
        class="h-10 px-3 sm:h-8">Requires it outright</Chip
      >
      {#if offered && termName}
        <Chip
          shape="pill"
          size="md"
          pressed={offeredOnly}
          onclick={() => (offeredOnly = !offeredOnly)}
          class="h-10 px-3 sm:h-8">Offered in {termName}</Chip
        >
      {/if}
    </div>
    {#if shown.length === 0}
      <p class="text-body-2 text-muted">Nothing matches these filters.</p>
    {/if}
    {#each groups as [prefix, list] (prefix)}
      {@const open = expanded.includes(prefix) || list.length <= GROUP_PREVIEW + 2}
      <section aria-label={prefixName(prefix)} class="min-w-0">
        <h4 class="mb-1 flex items-baseline gap-2 text-label-lg text-ink">
          {prefixName(prefix)}
          <span class="tabular font-normal text-muted">{list.length}</span>
        </h4>
        <ul class="flex flex-col">
          {#each open ? list : list.slice(0, GROUP_PREVIEW) as u (u.id)}
            {@const kind = kindOf(u)}
            <li class="flex min-w-0 items-center gap-2 border-t border-line py-1.5 first:border-t-0">
              <CourseChip code={codeOf(u.id)} href={href(u.id)} />
              <span class="min-w-0 flex-1 truncate text-body-2 text-ink">{title(u.id) ?? ""}</span>
              <span
                class="shrink-0 text-label-md {u.required ? 'text-strong' : 'text-muted'}"
                title={kind.help}>{kind.text}</span
              >
            </li>
          {/each}
        </ul>
        {#if !open}
          <button
            type="button"
            onclick={() => (expanded = [...expanded, prefix])}
            class="mt-1 min-h-10 text-label-lg text-accent-ink underline-offset-2 hover:underline sm:min-h-8"
          >
            Show all {list.length}
          </button>
        {/if}
      </section>
    {/each}
  </div>
{/if}
