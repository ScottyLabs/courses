<!--
  Search to add a course (Courseweave's AddCourseSearch): a combobox over the semester's courses. Arrowing or hovering
  a result previews its sections on the calendar; Enter or a click adds it with the first sections that fit.
-->
<script lang="ts">
  import { Check } from "@lucide/svelte";
  import SearchInput from "$lib/components/ui/SearchInput.svelte";
  import { panelSurface } from "$lib/components/ui/styles";
  import type { Term } from "$lib/core/data";
  import { unitsText } from "$lib/core/planner/format";
  import { searchCourses } from "$lib/core/search";
  import { fittingSelections, type ScheduleView } from "$lib/core/state/schedule-view";
  import type { Course } from "$lib/core/types";

  const LIMIT = 8;

  let {
    term,
    view,
    onadd,
    onpreview,
    placeholder = "Search courses by ID, description, name or keyword...",
    class: className = "",
  }: {
    term: Term;
    view: ScheduleView;
    /** Adds a course that isn't in the schedule yet. */
    onadd: (course: Course) => void;
    /** The highlighted result, to preview on the calendar; null when nothing is. */
    onpreview: (courseId: string | null) => void;
    placeholder?: string;
    class?: string;
  } = $props();

  const id = $props.id();
  let query = $state("");
  let open = $state(false);
  let active = $state(0);
  let root = $state<HTMLElement>();
  let input = $state<HTMLInputElement>();

  const results = $derived(query.trim() === "" ? [] : searchCourses(term.searchDocs, query, LIMIT));
  const showList = $derived(open && query.trim() !== "");

  function close(): void {
    open = false;
    onpreview(null);
  }

  function highlight(i: number): void {
    active = i;
    onpreview(results[i]?.id ?? null);
  }

  function add(course: Course): void {
    if (!view.hasCourse(course.id)) onadd(course);
    query = "";
    active = 0;
    onpreview(null);
    input?.focus();
  }

  function onkeydown(event: KeyboardEvent): void {
    if (results.length === 0) {
      if (event.key === "Escape" && query !== "") {
        event.preventDefault();
        query = "";
      }
      return;
    }
    if (event.key === "ArrowDown") {
      open = true;
      highlight((active + 1) % results.length);
    } else if (event.key === "ArrowUp") {
      open = true;
      highlight((active - 1 + results.length) % results.length);
    } else if (event.key === "Enter") {
      const course = results[Math.min(active, results.length - 1)];
      if (course !== undefined) add(course);
    } else if (event.key === "Escape") {
      if (showList) close();
      else query = "";
    } else return;
    event.preventDefault();
  }

  $effect(() => {
    if (!open) return;
    const onPointer = (event: PointerEvent): void => {
      if (root !== undefined && !root.contains(event.target as Node)) close();
    };
    document.addEventListener("pointerdown", onPointer);
    return () => document.removeEventListener("pointerdown", onPointer);
  });
</script>

<div bind:this={root} class="relative {className}">
  <SearchInput
    bind:value={query}
    bind:ref={input}
    label="Search courses to add"
    {placeholder}
    role="combobox"
    aria-expanded={showList}
    aria-controls="{id}-results"
    aria-activedescendant={showList && results.length > 0 ? `${id}-r-${active}` : undefined}
    aria-autocomplete="list"
    {onkeydown}
    onfocus={() => (open = true)}
    oninput={() => {
      open = true;
      active = 0;
      onpreview(null);
    }}
  />
  {#if showList}
    <ul
      id="{id}-results"
      role="listbox"
      aria-label="Matching courses"
      onmouseleave={() => onpreview(null)}
      class="{panelSurface} absolute top-full right-0 left-0 z-40 mt-1 max-h-[min(60dvh,26rem)] overflow-y-auto p-1"
    >
      {#if results.length === 0}
        <li class="px-3 py-3 text-body-2 text-muted">
          No courses match &ldquo;{query.trim()}&rdquo;. Try a number like 15-122 or a word from the title.
        </li>
      {:else}
        {#each results as course, i (course.id)}
          {@const added = view.hasCourse(course.id)}
          {@const fits = fittingSelections(course, view).length > 0}
          <li
            id="{id}-r-{i}"
            role="option"
            aria-selected={i === active}
            onmouseenter={() => highlight(i)}
            onmousedown={(event) => event.preventDefault()}
            onclick={() => add(course)}
            onkeydown={() => {}}
            class="flex min-h-11 cursor-pointer items-center gap-3 rounded-[6px] px-2.5 py-1.5 {i === active
              ? 'bg-paper'
              : ''}"
          >
            <span class="tabular w-16 shrink-0 text-label-lg text-strong">{course.code}</span>
            <span class="min-w-0 flex-1">
              <span class="block truncate text-body-2 text-ink">{course.title}</span>
              <span class="block text-label-md text-muted">{unitsText(course.units)}</span>
            </span>
            {#if added}
              <span class="flex shrink-0 items-center gap-1 text-label-md text-muted">
                <Check aria-hidden="true" class="size-3.5" />Added
              </span>
            {:else if fits}
              <span class="shrink-0 text-label-md text-ok">Fits</span>
            {:else}
              <span class="shrink-0 text-label-md text-danger">Conflicts</span>
            {/if}
          </li>
        {/each}
      {/if}
    </ul>
  {/if}
</div>
