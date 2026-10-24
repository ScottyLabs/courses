// The semester a page shows, from its `term` query parameter (`/schedules?term=F26`): the built semesters, the one
// resolved (the parameter, else the last one visited, else the newest), and its course data. A missing or unknown
// `term` is replaced in the address bar so links and reloads keep the semester.
import { goto } from "$app/navigation";
import { page } from "$app/state";
import { catalog, resource, type Resource } from "$lib/data";
import type { Term } from "$lib/core/data";
import type { TermIndexEntry } from "$lib/core/types";
import { device, resolveTerm } from "$lib/state";

export interface PageTerm {
  readonly terms: Resource<TermIndexEntry[]>;
  /** The semester shown; null until the semesters load (or when none is built). */
  readonly termId: string | null;
  /** Its courses and sections. */
  readonly term: Resource<Term>;
  /** The loaded term when it's the one shown (not the previous one while the next loads). */
  readonly current: Term | null;
  /** Shows another semester. */
  set(termId: string): void;
}

export function pageTerm(): PageTerm {
  const terms = resource(() => catalog.terms());
  const requested = $derived(page.url.searchParams.get("term"));
  const termId = $derived(
    terms.value === undefined ? null : resolveTerm(requested, terms.value, device.lastTerm),
  );
  const term = resource(() => (termId === null ? null : catalog.term(termId)));

  $effect(() => {
    if (termId === null || requested === termId) return;
    const url = new URL(page.url);
    url.searchParams.set("term", termId);
    void goto(url, { replaceState: true, keepFocus: true, noScroll: true });
  });

  return {
    terms,
    get termId() {
      return termId;
    },
    term,
    get current() {
      const value = term.value;
      return value !== undefined && value.meta.id === termId ? value : null;
    },
    set(next: string) {
      const url = new URL(page.url);
      url.searchParams.set("term", next);
      void goto(url, { keepFocus: true, noScroll: true });
    },
  };
}
