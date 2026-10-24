<script lang="ts" module>
  import type { CatalogIndex } from "$lib/core/catalog-types";
  import { buildSearchDocs, type SearchDoc } from "$lib/core/search";
  import type { Course } from "$lib/core/types";

  const docsCache = new WeakMap<CatalogIndex, SearchDoc[]>();

  /** Search docs for every catalog course, built once per catalog. */
  function catalogDocs(index: CatalogIndex): SearchDoc[] {
    let docs = docsCache.get(index);
    if (docs === undefined) {
      const courses: Course[] = Object.values(index.courses).map((e) => ({
        id: e.id,
        code: e.code,
        title: e.title,
        units: e.units,
        dept: "",
        level: "U",
        instructors: [],
        sections: [],
        crossListed: [],
      }));
      docs = buildSearchDocs(courses);
      docsCache.set(index, docs);
    }
    return docs;
  }
</script>

<script lang="ts">
  // Pick any course: the semester's courses first, then the rest of the catalog (courses not offered now). Shows the
  // chosen course with a clear button once picked. Follows the ARIA combobox pattern. Courseweave's CoursePicker in
  // features/record/shared.tsx.
  import { Search, X } from "@lucide/svelte";
  import { tick } from "svelte";
  import { searchCourses } from "$lib/core/search";
  import type { RecordData } from "./record-data.svelte";

  const LIMIT = 8;

  let {
    value = $bindable(null),
    label,
    data,
    placeholder = "Search: 15-122, calculus",
    onpick,
  }: {
    value?: string | null;
    /** Visible label above the field. */
    label: string;
    data: RecordData;
    placeholder?: string;
    /** After a course is picked or cleared. */
    onpick?: (courseId: string | null) => void;
  } = $props();

  const id = $props.id();
  let query = $state("");
  let open = $state(false);
  let active = $state(0);
  let input = $state<HTMLInputElement>();
  let clearButton = $state<HTMLButtonElement>();

  const results = $derived.by(() => {
    if (query.trim() === "") return [];
    const term = data.term;
    const offered = term === null ? [] : searchCourses(term.searchDocs, query, LIMIT);
    if (offered.length >= LIMIT || data.index === null) return offered;
    const seen = new Set(offered.map((c) => c.id));
    const more = searchCourses(catalogDocs(data.index), query, LIMIT).filter((c) => !seen.has(c.id));
    return [...offered, ...more].slice(0, LIMIT);
  });
  const showList = $derived(open && query.trim() !== "");

  async function pick(course: Course): Promise<void> {
    value = course.id;
    query = "";
    open = false;
    active = 0;
    onpick?.(course.id);
    await tick();
    clearButton?.focus();
  }

  async function clear(): Promise<void> {
    value = null;
    onpick?.(null);
    await tick();
    input?.focus();
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
      if (course !== undefined) void pick(course);
    } else return;
    open = true;
    event.preventDefault();
  }
</script>

<div class="flex min-w-0 flex-col">
  <span id="{id}-label" class="mb-1.5 block text-label-lg text-ink">{label}</span>
  {#if value !== null}
    {@const info = data.lookup(value)}
    <div
      role="group"
      aria-labelledby="{id}-label"
      class="flex h-10 min-w-0 items-center gap-2 rounded-field border border-line-strong bg-paper pr-1 pl-4"
    >
      <span class="tabular shrink-0 text-label-lg text-strong">{info.code}</span>
      <span class="min-w-0 flex-1 truncate text-body-2 text-ink">{info.title ?? "Not in the catalog"}</span>
      <button
        bind:this={clearButton}
        type="button"
        aria-label="Clear {label.toLowerCase()}: {info.code}"
        onclick={() => void clear()}
        class="grid size-8 shrink-0 place-items-center rounded-full text-muted hover:bg-hover hover:text-ink"
      >
        <X aria-hidden="true" class="size-4" />
      </button>
    </div>
  {:else}
    <div
      class="relative"
      onfocusout={(event) => {
        const next = event.relatedTarget as Node | null;
        if (next === null || !event.currentTarget.contains(next)) open = false;
      }}
    >
      <Search
        aria-hidden="true"
        class="pointer-events-none absolute top-1/2 left-3.5 size-5 -translate-y-1/2 text-muted"
      />
      <input
        bind:this={input}
        bind:value={query}
        type="text"
        role="combobox"
        aria-labelledby="{id}-label"
        aria-expanded={showList}
        aria-controls="{id}-list"
        aria-activedescendant={showList && results.length > 0 ? `${id}-option-${active}` : undefined}
        aria-autocomplete="list"
        autocomplete="off"
        spellcheck="false"
        {placeholder}
        oninput={() => {
          open = true;
          active = 0;
        }}
        onfocus={() => (open = true)}
        {onkeydown}
        class="h-10 w-full rounded-field border border-line-strong bg-surface pr-4 pl-11 text-body-2 text-ink transition-colors placeholder:text-muted hover:border-muted focus:border-accent"
      />
      {#if showList}
        <ul
          id="{id}-list"
          role="listbox"
          aria-label="Matching courses"
          class="absolute top-11 right-0 left-0 z-30 max-h-80 overflow-y-auto rounded-[8px] border border-line bg-surface p-1 text-ink shadow-popover"
        >
          {#each results as course, index (course.id)}
            <li
              id="{id}-option-{index}"
              role="option"
              aria-selected={index === active}
              onmouseenter={() => (active = index)}
              onmousedown={(event) => event.preventDefault()}
              onclick={() => void pick(course)}
              onkeydown={(event) => {
                // Options aren't focusable (the field keeps focus and moves through them with the arrow keys).
                if (event.key === "Enter") void pick(course);
              }}
              class="flex min-h-11 cursor-pointer items-center gap-2.5 rounded-tag px-2.5 py-1.5 {index === active
                ? 'bg-paper'
                : ''}"
            >
              <span class="tabular w-[52px] shrink-0 text-label-lg text-strong">{course.code}</span>
              <span class="min-w-0 flex-1 truncate text-body-2">{course.title}</span>
              {#if course.units !== null}
                <span class="shrink-0 text-label-md text-muted">{course.units} units</span>
              {/if}
            </li>
          {:else}
            <li class="px-3 py-3 text-body-2 text-muted">
              No courses match &ldquo;{query.trim()}&rdquo;. Try a number like 21-127.
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  {/if}
</div>
