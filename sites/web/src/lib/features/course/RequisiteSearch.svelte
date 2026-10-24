<script lang="ts">
  import SearchInput from "$lib/components/ui/SearchInput.svelte";
  import { listRow, panelSurface } from "$lib/components/ui/styles";
  import type { CatalogIndex } from "$lib/core/catalog-types";
  import type { Term } from "$lib/core/data";
  import { buildSearchDocs, searchCourses, type SearchDoc } from "$lib/core/search";
  import type { Course } from "$lib/core/types";

  let {
    index,
    term,
    onpick,
  }: {
    index: CatalogIndex | null;
    /** The semester whose courses rank first; null searches the catalog only. */
    term: Term | null;
    onpick: (courseId: string) => void;
  } = $props();

  const LIMIT = 8;
  const id = $props.id();
  let query = $state("");
  let open = $state(false);
  let active = $state(0);
  let root = $state<HTMLElement>();

  // Catalog courses the semester doesn't offer, so they can be explored too.
  const extraDocs = $derived.by((): SearchDoc[] => {
    if (!index) return [];
    const extra: Course[] = [];
    for (const e of Object.values(index.courses)) {
      if (term?.courseById.has(e.id) === true) continue;
      extra.push({
        id: e.id,
        code: e.code,
        title: e.title,
        units: e.units,
        dept: "",
        level: "U",
        instructors: [],
        sections: [],
        crossListed: [],
      });
    }
    return buildSearchDocs(extra);
  });
  const results = $derived.by(() => {
    if (!query.trim()) return [];
    // The semester's courses rank first among equal matches; the rest of the catalog follows.
    const offered = term ? searchCourses(term.searchDocs, query, LIMIT) : [];
    const more = offered.length < LIMIT ? searchCourses(extraDocs, query, LIMIT - offered.length) : [];
    return [...offered, ...more];
  });
  const showList = $derived(open && query.trim().length > 0);

  function pick(course: Course): void {
    onpick(course.id);
    query = "";
    open = false;
    active = 0;
  }

  function onkeydown(event: KeyboardEvent): void {
    if (event.key === "Escape") {
      if (showList) open = false;
      else query = "";
      event.preventDefault();
      return;
    }
    if (results.length === 0) return;
    if (event.key === "ArrowDown") active = (active + 1) % results.length;
    else if (event.key === "ArrowUp") active = (active - 1 + results.length) % results.length;
    else if (event.key === "Enter") {
      const course = results[Math.min(active, results.length - 1)];
      if (course) pick(course);
    } else return;
    open = true;
    event.preventDefault();
  }
</script>

<div
  bind:this={root}
  class="relative"
  onfocusout={(event) => {
    if (!(event.relatedTarget instanceof Node && root?.contains(event.relatedTarget))) open = false;
  }}
>
  <SearchInput
    variant="field"
    bind:value={() => query, (next) => {
      query = next;
      open = true;
      active = 0;
    }}
    label="Find a course to explore"
    placeholder="Find a course: 15-213, linear algebra"
    role="combobox"
    aria-expanded={showList}
    aria-controls="{id}-results"
    aria-activedescendant={showList && results.length > 0 ? `${id}-r-${active}` : undefined}
    aria-autocomplete="list"
    onfocus={() => (open = true)}
    {onkeydown}
  />
  {#if showList}
    <ul
      id="{id}-results"
      role="listbox"
      aria-label="Matching courses"
      class="absolute top-12 right-0 left-0 z-40 overflow-hidden py-1 {panelSurface}"
    >
      {#if results.length === 0}
        <li class="px-3.5 py-3 text-body-2 text-muted">
          No courses match "{query.trim()}". Try a number like 15-213 or a word from the title.
        </li>
      {:else}
        {#each results as course, i (course.id)}
          <li
            id="{id}-r-{i}"
            role="option"
            aria-selected={i === active}
            data-active={i === active}
            onmouseenter={() => (active = i)}
            onmousedown={(event) => event.preventDefault()}
            onclick={() => pick(course)}
            onkeydown={() => {}}
            class="{listRow} min-h-11"
          >
            <span class="tabular w-14 shrink-0 font-semibold text-strong">{course.code}</span>
            <span class="min-w-0 flex-1 truncate">{course.title}</span>
            {#if term && !term.courseById.has(course.id)}
              <span class="shrink-0 text-label-md text-muted">Not in {term.meta.name}</span>
            {/if}
          </li>
        {/each}
      {/if}
    </ul>
  {/if}
</div>
