// Everything the instructor pages share: the semester, the list's filters (kept in the URL), the loaded catalog
// documents, the results, and the instructor being shown. The layout at routes/instructors creates it once, so the
// list keeps its state while the student opens instructors, and the pages read it from context.
import { getContext, setContext, untrack } from "svelte";
import { MediaQuery } from "svelte/reactivity";
import { goto } from "$app/navigation";
import { page } from "$app/state";
import { aggregateFce, selectFceRows } from "$lib/components/course/fce-aggregate";
import { fceOptions } from "$lib/components/course/fce-options.svelte";
import { loadInstructorFce } from "$lib/components/instructor/instructor-fce";
import { getSession } from "$lib/components/shell/session";
import { catalog, resource } from "$lib/data";
import { lastName, type Instructor } from "$lib/core/data";
import { departmentName } from "$lib/core/departments";
import type { AllInstructor } from "$lib/core/instructors/allInstructors";
import { scoreInstructor } from "$lib/core/instructors/instructorData";
import { saved, device } from "$lib/state";
import { ALL_SEMESTERS, isAllSemesters, resolveTerm } from "$lib/state/term-scope";
import { loadAllInstructors, loadCourseLookup, loadInstructorList } from "./data";
import type { ListedDoc } from "./list";

export type Sort = "name" | "sections";

/** The cards/list switch (ViewToggle's ListView). */
export type View = "cards" | "list";

const KEY = Symbol("instructors");

function byName(a: Instructor, b: Instructor): number {
  return lastName(a.name).localeCompare(lastName(b.name)) || a.name.localeCompare(b.name);
}

export class InstructorsScope {
  readonly #session = getSession();
  readonly signedIn = $derived(this.#session.status === "signed-in");
  readonly sessionLoading = $derived(this.#session.status === "loading");

  // ---- The URL: semester, filters and the instructor shown ----

  readonly #params = $derived(page.url.searchParams);
  readonly termParam = $derived(this.#params.get("term"));
  readonly allMode = $derived(isAllSemesters(this.termParam));
  readonly dept = $derived(this.#params.get("dept") ?? "");
  readonly sort = $derived<Sort>(this.#params.get("sort") === "sections" ? "sections" : "name");
  readonly savedOnly = $derived(this.#params.get("saved") === "1");
  readonly #urlQuery = $derived(this.#params.get("q") ?? "");
  /** The open instructor's slug, on `/instructors/<slug>`. */
  readonly slug = $derived(page.params.id ?? null);

  // ---- Catalog documents ----

  readonly terms = resource(() => catalog.terms());
  /** The real semester shown (the last visited, or the newest, when the URL names none). */
  readonly termId = $derived(
    this.terms.value ? resolveTerm(this.termParam, this.terms.value, device.lastTerm) : null,
  );
  readonly termMeta = $derived(this.terms.value?.find((t) => t.id === this.termId));
  /**
   * The list: one small document for the semester or for all semesters (decision 43). The full documents below load
   * only while an instructor is open.
   */
  readonly list = resource(() =>
    this.signedIn && (this.allMode || this.termId !== null)
      ? loadInstructorList(this.allMode ? null : this.termId)
      : null,
  );
  readonly #open = $derived(this.signedIn && this.slug !== null);
  readonly term = resource(() =>
    this.#open && !this.allMode && this.termId !== null ? catalog.term(this.termId) : null,
  );
  readonly all = resource(() => (this.#open && this.allMode ? loadAllInstructors() : null));
  /** Course titles for the open instructor's evaluation history. */
  readonly lookup = resource(() => (this.#open ? loadCourseLookup() : null));

  /** `term` for links: "all" or the semester shown. */
  readonly linkTerm = $derived(this.allMode ? ALL_SEMESTERS : this.termId);
  readonly scopeName = $derived(
    this.allMode ? "all semesters" : (this.termMeta?.name ?? "this semester"),
  );
  readonly forbidden = $derived(
    this.list.status === "forbidden" ||
      this.term.status === "forbidden" ||
      this.all.status === "forbidden",
  );
  readonly failed = $derived(
    this.list.status === "error" ||
      this.term.status === "error" ||
      this.all.status === "error" ||
      this.terms.status === "error",
  );
  /** The list has loaded for the semester shown. */
  readonly ready = $derived(this.list.value !== undefined && this.list.status !== "loading");
  /** The open instructor's documents have loaded. */
  readonly detailReady = $derived(
    this.allMode
      ? this.all.value !== undefined && this.all.status !== "loading"
      : this.term.value !== undefined && this.term.status !== "loading",
  );
  /** "All semesters" can't be built without the teaching document. */
  readonly noTeaching = $derived(
    this.allMode && this.list.status === "ready" && this.list.value === null,
  );

  // ---- The list ----

  /** The search box, typed into here and written to the URL after a pause. */
  query = $state(untrack(() => this.#urlQuery));
  #written = untrack(() => this.#urlQuery);
  /** The cards/list switch; the layout passes the device's last choice. */
  view = $state<View>("cards");

  readonly docs = $derived<ListedDoc[]>(this.list.value ?? []);
  readonly total = $derived(this.docs.length);
  readonly deptOptions = $derived(
    [...new Set(this.docs.flatMap((doc) => doc.inst.depts))]
      .map((code) => ({ value: code, label: departmentName(code), keywords: code }))
      .toSorted((a, b) => a.label.localeCompare(b.label)),
  );
  readonly results = $derived.by((): ListedDoc[] => {
    const q = this.query;
    const savedIds = new Set(saved.ids);
    const scored: { doc: ListedDoc; score: number }[] = [];
    for (const doc of this.docs) {
      if (this.dept !== "" && !doc.inst.depts.includes(this.dept)) continue;
      if (this.savedOnly && !doc.inst.courseIds.some((id) => savedIds.has(id))) continue;
      const score = scoreInstructor(doc, q);
      if (score > 0) scored.push({ doc, score });
    }
    // With a query, name matches (100 and up) rank above course matches; then the chosen order applies.
    const tier = (score: number): number => (q.trim() !== "" && score < 100 ? 1 : 0);
    return scored
      .toSorted(
        (a, b) =>
          tier(a.score) - tier(b.score) ||
          (this.sort === "sections" ? b.doc.inst.sectionCount - a.doc.inst.sectionCount : 0) ||
          byName(a.doc.inst, b.doc.inst),
      )
      .map((item) => item.doc);
  });
  /** Letter headings in the rows: sorted by name, without a query. */
  readonly grouped = $derived(this.sort === "name" && this.query.trim() === "");
  readonly filterCount = $derived(
    (this.dept === "" ? 0 : 1) + (this.savedOnly ? 1 : 0) + (this.sort === "sections" ? 1 : 0),
  );
  readonly countText = $derived(
    this.results.length === this.total
      ? `${this.total.toLocaleString("en-US")} instructors`
      : `${this.results.length.toLocaleString("en-US")} of ${this.total.toLocaleString("en-US")} instructors`,
  );

  // ---- The instructor shown ----

  readonly selected = $derived.by((): Instructor | AllInstructor | undefined => {
    if (this.slug === null) return undefined;
    return this.allMode
      ? this.all.value?.bySlug.get(this.slug)
      : this.term.value?.instructorBySlug.get(this.slug);
  });
  readonly #wide = new MediaQuery("(min-width: 80rem)");
  /** Rows beside the open instructor (Courseweave's split), in the list view on wide screens. */
  readonly split = $derived(this.slug !== null && this.view === "list" && this.#wide.current);
  readonly fce = resource(() =>
    this.selected === undefined ? null : loadInstructorFce(catalog, this.selected.name),
  );
  /** Their courses the aggregate counts; none chosen counts every course. */
  courses = $state<string[]>([]);
  /** The section table shows every semester, not just the aggregate's. */
  everySemester = $state(false);
  readonly windowed = $derived(
    this.fce.value ? selectFceRows(this.fce.value.rows, fceOptions.value, this.courses) : [],
  );
  readonly stats = $derived(this.fce.value ? aggregateFce(this.windowed) : null);
  readonly courseOptions = $derived(
    (this.fce.value?.summary.courses ?? [])
      .map((course) => {
        const title = this.titleOf(course.courseId);
        return {
          value: course.courseId,
          label: `${course.courseId.slice(0, 2)}-${course.courseId.slice(2)}`,
          description: title,
          keywords: title,
        };
      })
      .toSorted((a, b) => a.label.localeCompare(b.label)),
  );

  constructor(view: View) {
    this.view = view;
    // Back, Forward or a link changed the URL's query: follow it.
    $effect(() => {
      const next = this.#urlQuery;
      if (next !== this.#written) {
        this.#written = next;
        this.query = next;
      }
    });
    $effect(() => {
      const next = this.query;
      const timer =
        next === untrack(() => this.#urlQuery)
          ? null
          : setTimeout(() => {
              this.#written = next;
              this.setParams({ q: next.trim() === "" ? null : next });
            }, 250);
      return () => {
        if (timer !== null) clearTimeout(timer);
      };
    });
    // A different instructor starts with all of their courses counted.
    $effect(() => {
      void this.slug;
      untrack(() => {
        this.courses = [];
      });
    });
  }

  /** Changes query parameters in place (no new history entry). */
  setParams(patch: Record<string, string | null>): void {
    const url = new URL(page.url);
    for (const [key, value] of Object.entries(patch)) {
      if (value === null || value === "") url.searchParams.delete(key);
      else url.searchParams.set(key, value);
    }
    void goto(url, { replaceState: true, keepFocus: true, noScroll: true });
  }

  /** Clears the search and the list's filters. */
  reset(): void {
    this.query = "";
    this.#written = "";
    this.setParams({ q: null, dept: null, saved: null, sort: null });
  }

  /** The list's own query string, carried to instructor pages and back. */
  get search(): string {
    const params = new URLSearchParams(this.#params);
    if (this.linkTerm !== null) params.set("term", this.linkTerm);
    const text = params.toString();
    return text === "" ? "" : `?${text}`;
  }

  /** An instructor's page, keeping the list's filters. */
  href(slug: string): string {
    return `/instructors/${encodeURIComponent(slug)}${this.search}`;
  }

  /** The list, with its filters. */
  get listHref(): string {
    return `/instructors${this.search}`;
  }

  /** A course's title, when the catalog knows it. */
  titleOf(courseId: string): string | undefined {
    return this.lookup.value?.title(courseId) ?? this.term.value?.courseById.get(courseId)?.title;
  }
}

/** Creates the instructor pages' state; call once, in the instructors layout. */
export function provideInstructorsScope(view: View): InstructorsScope {
  return setContext(KEY, new InstructorsScope(view));
}

export function useInstructorsScope(): InstructorsScope {
  const scope = getContext<InstructorsScope | undefined>(KEY);
  if (scope === undefined) throw new Error("useInstructorsScope() outside routes/instructors");
  return scope;
}
