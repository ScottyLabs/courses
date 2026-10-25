<script lang="ts">
  // "+ Add a course" at the bottom of a semester: a combobox over the whole catalog that shows whether each course
  // is offered that semester. Type, Enter, type the next. Courseweave's features/degree/AddCourseInput.tsx.
  import { Plus } from "@lucide/svelte";
  import { findCourses } from "$lib/components/course/course-search";
  import type { CatalogIndex } from "$lib/core/catalog-types";
  import { offering, OFFERING_LABEL, type Offering } from "$lib/core/plan";
  import { termLabel } from "$lib/core/requisites";
  import type { SearchDoc } from "$lib/core/search";

  let {
    term,
    catalog,
    termDocs,
    scheduled,
    inPlan,
    onadd,
  }: {
    /** The semester courses are added to; drives the offering hints. */
    term: string;
    catalog: CatalogIndex | null;
    /** The browsed semester's courses, for listings the catalog doesn't have. */
    termDocs: SearchDoc[];
    /** Course ids on that semester's published schedule, when it has one. */
    scheduled: Set<string> | undefined;
    /** Course ids already in the plan, marked in the results. */
    inPlan: Set<string>;
    onadd: (courseId: string) => void;
  } = $props();

  const id = $props.id();
  let query = $state("");
  let active = $state(0);
  let root = $state<HTMLElement>();

  const results = $derived(findCourses(catalog, termDocs, query));
  const showList = $derived(query.trim() !== "");
  const TONE: Record<Offering, string> = {
    scheduled: "text-ok",
    "not-scheduled": "text-warn",
    usually: "text-muted",
    "other-seasons": "text-warn",
    unknown: "text-muted",
  };

  function add(courseId: string): void {
    onadd(courseId);
    query = "";
    active = 0;
  }

  function onkeydown(event: KeyboardEvent): void {
    if (event.key === "Escape") {
      if (query !== "") {
        event.preventDefault();
        event.stopPropagation();
        query = "";
      }
      return;
    }
    if (results.length === 0) return;
    if (event.key === "ArrowDown") active = (active + 1) % results.length;
    else if (event.key === "ArrowUp") active = (active - 1 + results.length) % results.length;
    else if (event.key === "Enter") {
      const course = results[Math.min(active, results.length - 1)];
      if (course) add(course.id);
    } else return;
    event.preventDefault();
  }

  function onfocusout(event: FocusEvent): void {
    // Leaving the field with a query left in it keeps it; the list hides with the focus.
    if (!root?.contains(event.relatedTarget as Node | null)) active = 0;
  }
</script>

<div bind:this={root} class="relative" {onfocusout}>
  <label
    class="focus-field flex min-h-10 cursor-text items-center gap-2 rounded-field border border-transparent px-2 text-ink focus-within:bg-paper hover:bg-paper md:min-h-8"
  >
    <Plus aria-hidden="true" class="size-4 shrink-0" />
    <span class="sr-only">Add a course to {termLabel(term)}</span>
    <input
      bind:value={query}
      oninput={() => (active = 0)}
      {onkeydown}
      placeholder="Add a course"
      autocomplete="off"
      spellcheck="false"
      role="combobox"
      aria-expanded={showList}
      aria-controls="{id}-list"
      aria-activedescendant={showList && results.length > 0 ? `${id}-${active}` : undefined}
      aria-autocomplete="list"
      class="h-10 min-w-0 flex-1 bg-transparent text-label-lg text-strong outline-none placeholder:text-ink md:h-8"
    />
  </label>
  {#if showList}
    <ul
      id="{id}-list"
      role="listbox"
      aria-label="Matching courses"
      class="scrollbar-thin absolute top-full right-0 left-0 z-40 mt-1 max-h-80 overflow-y-auto rounded-[8px] border border-line bg-surface py-1 text-ink shadow-popover"
    >
      {#if results.length === 0}
        <li class="px-3 py-3 text-body-2 text-muted">No courses match "{query.trim()}".</li>
      {:else}
        {#each results as course, i (course.id)}
          {@const o = offering(course.id, term, catalog, scheduled)}
          <!-- The combobox field handles the keyboard (aria-activedescendant); clicks are for the mouse. -->
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <li
            id="{id}-{i}"
            role="option"
            aria-selected={i === active}
            onmouseenter={() => (active = i)}
            onmousedown={(event) => event.preventDefault()}
            onclick={() => add(course.id)}
            class="flex min-h-11 cursor-pointer flex-col justify-center px-3 py-1.5 {i === active ? 'bg-paper' : ''}"
          >
            <span class="flex min-w-0 items-baseline gap-2">
              <span class="tabular shrink-0 text-label-lg text-strong">{course.code}</span>
              <span class="min-w-0 flex-1 truncate text-body-2 text-ink">{course.title}</span>
              {#if course.units !== null}<span class="tabular shrink-0 text-label-md text-muted">{course.units}u</span>{/if}
            </span>
            <span class="flex gap-2 text-label-md font-normal">
              <span class={TONE[o]}>{OFFERING_LABEL[o]}</span>
              {#if inPlan.has(course.id)}<span class="text-muted">Already in your plan</span>{/if}
            </span>
          </li>
        {/each}
      {/if}
    </ul>
  {/if}
</div>
