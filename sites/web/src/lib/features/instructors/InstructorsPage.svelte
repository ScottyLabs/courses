<script lang="ts">
  import { untrack } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { Bookmark, GraduationCap, SearchX, ArrowDownUp, UserX } from "@lucide/svelte";
  import PageToolbar from "$lib/components/layout/PageToolbar.svelte";
  import RailGroup from "$lib/components/layout/RailGroup.svelte";
  import RailLayout from "$lib/components/layout/RailLayout.svelte";
  import RailSection from "$lib/components/layout/RailSection.svelte";
  import { getSession } from "$lib/components/shell/session";
  import SignInPrompt from "$lib/components/shell/SignInPrompt.svelte";
  import Title from "$lib/components/shell/Title.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Chip from "$lib/components/ui/Chip.svelte";
  import Combobox from "$lib/components/ui/Combobox.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import SearchInput from "$lib/components/ui/SearchInput.svelte";
  import SegmentedControl from "$lib/components/ui/SegmentedControl.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import Skeleton from "$lib/components/ui/Skeleton.svelte";
  import Spinner from "$lib/components/ui/Spinner.svelte";
  import Switch from "$lib/components/ui/Switch.svelte";
  import ViewToggle, { storedView, type ListView } from "$lib/components/ui/ViewToggle.svelte";
  import InstructorCard from "$lib/components/instructor/InstructorCard.svelte";
  import InstructorRow from "$lib/components/instructor/InstructorRow.svelte";
  import { courseCode } from "$lib/components/instructor/format";
  import { catalog, resource } from "$lib/data";
  import { lastName, type Instructor } from "$lib/core/data";
  import { departmentName } from "$lib/core/departments";
  import type { AllInstructor } from "$lib/core/instructors/allInstructors";
  import { buildInstructorDocs, scoreInstructor, type InstructorDoc } from "$lib/core/instructors/instructorData";
  import { coursePath, instructorPath } from "$lib/core/routes";
  import { device, saved } from "$lib/state";
  import { ALL_SEMESTERS, isAllSemesters, resolveTerm } from "$lib/state/term-scope";
  import AggregateFceSettings from "./AggregateFceSettings.svelte";
  import { loadAllInstructors, semesterOptions, teachingSummary } from "./data";
  import { recentTeachingRate } from "./fce-aggregate";

  type Sort = "name" | "sections";

  const PAGE = { cards: 12, list: 80 };
  const session = getSession();
  const signedIn = $derived(session.status === "signed-in");

  // Filters live in the URL so Back from an instructor, a reload or a shared link shows the same list.
  const params = $derived(page.url.searchParams);
  const termParam = $derived(params.get("term"));
  const allMode = $derived(isAllSemesters(termParam));
  const dept = $derived(params.get("dept") ?? "");
  const sort = $derived<Sort>(params.get("sort") === "sections" ? "sections" : "name");
  const savedOnly = $derived(params.get("saved") === "1");
  const urlQuery = $derived(params.get("q") ?? "");

  const terms = resource(() => catalog.terms());
  const termId = $derived(terms.value ? resolveTerm(termParam, terms.value, device.lastTerm) : null);
  const termMeta = $derived(terms.value?.find((t) => t.id === termId));
  const term = resource(() => (signedIn && !allMode && termId !== null ? catalog.term(termId) : null));
  const all = resource(() => (signedIn && allMode ? loadAllInstructors() : null));
  // The FCE index gives the list rows their teaching rating and tells a non-CMU account apart.
  const fceIndex = resource(() => (signedIn ? catalog.fceInstructors() : null));

  let view = $state<ListView>(storedView("instructors") ?? "cards");

  // The search box is typed into locally and written to the URL after a pause.
  let query = $state(untrack(() => urlQuery));
  let written = untrack(() => urlQuery);
  $effect(() => {
    // Back, Forward or a link changed the URL: follow it.
    const next = urlQuery;
    if (next !== written) {
      written = next;
      query = next;
    }
  });
  $effect(() => {
    const next = query;
    if (next === untrack(() => urlQuery)) return;
    const timer = setTimeout(() => {
      written = next;
      setParams({ q: next.trim() === "" ? null : next });
    }, 250);
    return () => clearTimeout(timer);
  });

  function setParams(patch: Record<string, string | null>): void {
    const url = new URL(page.url);
    for (const [key, value] of Object.entries(patch)) {
      if (value === null || value === "") url.searchParams.delete(key);
      else url.searchParams.set(key, value);
    }
    void goto(url, { replaceState: true, keepFocus: true, noScroll: true });
  }

  const docs = $derived.by((): InstructorDoc[] => {
    if (allMode) return all.value?.docs ?? [];
    return term.value ? buildInstructorDocs(term.value) : [];
  });
  const total = $derived(docs.length);
  const ready = $derived(allMode ? all.value !== undefined && all.status !== "loading" : term.value !== undefined);

  const deptOptions = $derived.by(() => {
    const used = new Set(docs.flatMap((doc) => doc.inst.depts));
    return [...used]
      .map((code) => ({ value: code, label: departmentName(code), keywords: code }))
      .toSorted((a, b) => a.label.localeCompare(b.label));
  });

  const savedIds = $derived(new Set(saved.ids));

  const results = $derived.by(() => {
    const q = query;
    const scored: { doc: InstructorDoc; score: number }[] = [];
    for (const doc of docs) {
      if (dept !== "" && !doc.inst.depts.includes(dept)) continue;
      if (savedOnly && !doc.inst.courseIds.some((id) => savedIds.has(id))) continue;
      const score = scoreInstructor(doc, q);
      if (score > 0) scored.push({ doc, score });
    }
    const byName = (a: Instructor, b: Instructor): number =>
      lastName(a.name).localeCompare(lastName(b.name)) || a.name.localeCompare(b.name);
    // With a query, name matches (100 and up) rank above course matches; then the chosen order applies.
    const tier = (score: number): number => (q.trim() !== "" && score < 100 ? 1 : 0);
    return scored
      .toSorted(
        (a, b) =>
          tier(a.score) - tier(b.score) ||
          (sort === "sections" ? b.doc.inst.sectionCount - a.doc.inst.sectionCount : 0) ||
          byName(a.doc.inst, b.doc.inst),
      )
      .map((item) => item.doc);
  });
  const grouped = $derived(sort === "name" && query.trim() === "");

  // Incremental rendering: more appear as the end of the list comes into view.
  let limit = $state(PAGE.cards);
  $effect(() => {
    void [query, dept, sort, savedOnly, termParam, view];
    limit = untrack(() => PAGE[view]);
  });
  const visible = $derived(results.slice(0, limit));

  function more(element: HTMLElement): () => void {
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting)) limit += PAGE[view];
      },
      { rootMargin: "800px 0px" },
    );
    observer.observe(element);
    return () => observer.disconnect();
  }

  const filterCount = $derived((dept === "" ? 0 : 1) + (savedOnly ? 1 : 0) + (sort === "sections" ? 1 : 0));
  const termName = $derived(allMode ? "all semesters" : (termMeta?.name ?? "this semester"));

  function reset(): void {
    query = "";
    written = "";
    setParams({ q: null, dept: null, saved: null, sort: null });
  }

  function href(doc: InstructorDoc): string {
    return instructorPath(doc.inst.slug, allMode ? ALL_SEMESTERS : termId);
  }

  function rating(name: string): number | null {
    const entry = fceIndex.value?.get(name);
    return entry === undefined ? null : recentTeachingRate(entry);
  }

  function countText(inst: Instructor): string {
    const n = sort === "sections" ? inst.sectionCount : inst.courseIds.length;
    const unit = sort === "sections" ? "section" : "course";
    return `${n} ${unit}${n === 1 ? "" : "s"}`;
  }

  let searchField = $state<HTMLInputElement>();
  function onkeydown(event: KeyboardEvent): void {
    if (event.key !== "/" || event.metaKey || event.ctrlKey || event.altKey) return;
    const target = event.target;
    if (
      target instanceof HTMLElement &&
      (target.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName))
    )
      return;
    event.preventDefault();
    searchField?.focus();
    searchField?.select();
  }

  function openFirst(): void {
    const [first] = results;
    if (first !== undefined) void goto(href(first));
  }
</script>

<svelte:window {onkeydown} />

{#snippet semesterPicker(width: string, variant: "field" | "compact")}
  {#if terms.value}
    <Select
      label="Semester"
      {variant}
      options={semesterOptions(terms.value)}
      value={allMode ? ALL_SEMESTERS : (termId ?? "")}
      onchange={(event) => setParams({ term: event.currentTarget.value })}
      class={width}
    />
  {/if}
{/snippet}

<Title value={allMode ? "Instructors, all semesters" : termMeta ? `Instructors, ${termMeta.name}` : "Instructors"} />

<RailLayout storageKey="instructors" {filterCount} backLabel="Instructors">
  {#snippet rail()}
    <RailGroup>
      <RailSection title="Department" icon={GraduationCap} badge={dept === "" ? 0 : 1}>
        <Combobox
          options={deptOptions}
          value={dept === "" ? null : dept}
          label="Department"
          placeholder="Any department"
          onchange={(value) => setParams({ dept: value })}
        />
      </RailSection>
      <RailSection title="Saved courses" icon={Bookmark} badge={savedOnly ? 1 : 0}>
        <Switch
          label="Only instructors teaching my saved courses"
          checked={savedOnly}
          disabled={saved.ids.length === 0}
          labelPosition="end"
          onchange={(on) => setParams({ saved: on ? "1" : null })}
        />
        {#if saved.ids.length === 0}
          <p class="text-label-md font-normal text-muted">Save courses on the Courses page to use this filter.</p>
        {/if}
      </RailSection>
      <RailSection title="Sort by" icon={ArrowDownUp}>
        <SegmentedControl
          label="Sort by"
          options={[
            { value: "name", label: "Last name" },
            { value: "sections", label: "Most sections" },
          ]}
          value={sort}
          onchange={(value) => setParams({ sort: value === "sections" ? "sections" : null })}
        />
      </RailSection>
      <Button variant="secondary" full onclick={reset}>Reset</Button>
    </RailGroup>
    <RailGroup title="Aggregate FCEs">
      <AggregateFceSettings />
    </RailGroup>
  {/snippet}

  {#snippet header()}
    <PageToolbar
      title="Instructors"
      hideTitle="desktop"
      count={signedIn && ready ? (results.length === total ? `${total.toLocaleString("en-US")} instructors` : `${results.length.toLocaleString("en-US")} of ${total.toLocaleString("en-US")} instructors`) : undefined}
      onreset={filterCount > 0 || query !== "" ? reset : undefined}
    >
      {#snippet search()}
        <SearchInput
          bind:value={query}
          bind:ref={searchField}
          label="Search instructors"
          placeholder="Search instructors by name, course number or course title&hellip;"
          onsubmit={openFirst}
        />
      {/snippet}
      {#snippet chips()}
        {#if dept !== ""}
          <Chip label={departmentName(dept)} onremove={() => setParams({ dept: null })} />
        {/if}
        {#if savedOnly}
          <Chip label="Saved courses" onremove={() => setParams({ saved: null })} />
        {/if}
        {#if sort === "sections"}
          <Chip label="Most sections" onremove={() => setParams({ sort: null })} />
        {/if}
      {/snippet}
      {#snippet intro()}
        <!-- Phones: the semester gets its own row; wider screens have it beside the view switch. -->
        <div class="mb-2 lg:hidden">{@render semesterPicker("w-full", "field")}</div>
      {/snippet}
      {#snippet actions()}
        <div class="hidden lg:block">{@render semesterPicker("w-44", "compact")}</div>
        <ViewToggle pageKey="instructors" bind:value={view} />
      {/snippet}
    </PageToolbar>
  {/snippet}

  {#if session.status === "loading"}
    <div class="flex justify-center py-16"><Spinner size="lg" label="Checking your session" /></div>
  {:else if !signedIn}
    <SignInPrompt
      title="Sign in to see instructors"
      description="Who teaches each course and their course evaluations come from CMU systems, so they're only shown to signed-in CMU students."
    />
  {:else if fceIndex.status === "forbidden" || term.status === "forbidden" || all.status === "forbidden"}
    <EmptyState
      icon={UserX}
      title="Instructors need a CMU account"
      description="You're signed in with an account that isn't a CMU one. Sign out, then sign in with your Andrew ID."
    />
  {:else if term.status === "error" || all.status === "error" || terms.status === "error"}
    <EmptyState
      icon={SearchX}
      title="Instructors didn't load"
      description="The course catalog couldn't be reached. Check your connection, then reload the page."
    >
      <Button onclick={() => location.reload()}>Reload</Button>
    </EmptyState>
  {:else if allMode && all.status === "ready" && all.value === null}
    <EmptyState
      icon={SearchX}
      title="All semesters isn't available"
      description="This catalog has no list of who teaches what across semesters. Pick a semester instead."
    />
  {:else if !ready}
    <div aria-busy="true" aria-live="polite" class="flex flex-col gap-4">
      <span class="sr-only">Loading instructors</span>
      {#each [0, 1, 2] as index (index)}
        <Skeleton class="h-48 w-full rounded-card" />
      {/each}
    </div>
  {:else if results.length === 0}
    <EmptyState
      icon={SearchX}
      title="No instructors match"
      description={total === 0
        ? `No instructors are listed for ${termName} yet.`
        : "Try a last name, a course number like 15-122, or clear the filters."}
    >
      {#if filterCount > 0 || query !== ""}<Button onclick={reset}>Clear filters</Button>{/if}
    </EmptyState>
  {:else if view === "cards"}
    <ul class="flex flex-col gap-6" aria-label="Instructors">
      {#each visible as doc (doc.inst.slug)}
        <li>
          <InstructorCard name={doc.inst.name} depts={doc.inst.depts} href={href(doc)}>
            {#snippet meta()}
              {#if allMode}
                <p class="mt-2 text-label-md font-normal text-muted">
                  {teachingSummary((doc.inst as AllInstructor).terms)}
                </p>
              {:else}
                <ul class="mt-3 flex flex-wrap gap-1.5" aria-label="Teaching in {termName}">
                  {#each doc.inst.courseIds.toSorted().slice(0, 8) as id (id)}
                    <li><Chip size="sm" href={coursePath(id, termId)} label={courseCode(id)} /></li>
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
    <ul class="overflow-clip rounded-card border border-pressed bg-surface" aria-label="Instructors">
      {#each visible as doc, index (doc.inst.slug)}
        {#if grouped && (index === 0 || visible[index - 1]?.letter !== doc.letter)}
          <li
            aria-hidden="true"
            class="sticky top-(--header-h) z-[1] border-b border-line bg-surface-2 px-4 py-1 text-label-md text-muted sm:px-5"
          >
            {doc.letter}
          </li>
        {/if}
        <li class="border-b border-line last:border-b-0">
          <InstructorRow
            name={doc.inst.name}
            href={href(doc)}
            depts={doc.inst.depts}
            detail={allMode ? teachingSummary((doc.inst as AllInstructor).terms) : undefined}
            count={countText(doc.inst)}
            rating={rating(doc.inst.name)}
          />
        </li>
      {/each}
    </ul>
  {/if}

  {#if signedIn && ready && visible.length < results.length}
    <div {@attach more} class="mt-6 flex justify-center">
      <Button onclick={() => (limit += PAGE[view])}>Show more instructors</Button>
    </div>
  {/if}
</RailLayout>
