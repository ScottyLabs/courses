// The Courses page's query, kept in the URL so searches can be shared and Back works. Changes write the URL after
// a short pause (replacing the history entry); a semester change or opening a course adds an entry. Navigation from
// elsewhere (a link with filters, Back) is read back in.
import { goto } from "$app/navigation";
import { page } from "$app/state";
import { parseExplorer, writeExplorer, type ExplorerQuery } from "./query";

const WRITE_DELAY = 300;
const SEARCH_DELAY = 140;

export class ExplorerUrl {
  query = $state<ExplorerQuery>(parseExplorer(page.url.searchParams));
  /** The search field's text; the query follows it after a pause, so typing stays responsive. */
  searchText = $state("");
  #written: string;
  #timer: ReturnType<typeof setTimeout> | undefined;

  constructor() {
    this.searchText = this.query.q;
    this.#written = writeExplorer(this.query).toString();
  }

  /** Reads a URL change made elsewhere. Call from an $effect that reads `page.url`. */
  sync(params: URLSearchParams): void {
    const incoming = parseExplorer(params);
    const search = writeExplorer(incoming).toString();
    if (search === this.#written) return;
    this.#written = search;
    this.query = incoming;
    this.searchText = incoming.q;
  }

  /** Changes the query; with `push` the URL changes at once, as a new history entry. */
  update(next: ExplorerQuery, push = false): void {
    this.query = next;
    clearTimeout(this.#timer);
    if (push) this.#write(true);
    else
      this.#timer = setTimeout(() => {
        this.#write(false);
      }, WRITE_DELAY);
  }

  /** Starts the search a moment after the last keystroke. Call from an $effect; returns its cleanup. */
  followSearch(): () => void {
    const text = this.searchText;
    const handle =
      text === this.query.q
        ? undefined
        : setTimeout(() => {
            this.update({ ...this.query, q: text });
          }, SEARCH_DELAY);
    return () => {
      clearTimeout(handle);
    };
  }

  /** A link to this page with the query changed, such as one opening a course in the detail panel. */
  href(patch: Partial<ExplorerQuery>): string {
    const search = writeExplorer({ ...this.query, q: this.searchText, ...patch }).toString();
    return search === "" ? "/" : `/?${search}`;
  }

  destroy(): void {
    clearTimeout(this.#timer);
  }

  #write(push: boolean): void {
    clearTimeout(this.#timer);
    const search = writeExplorer(this.query).toString();
    if (search === this.#written) return;
    this.#written = search;
    void goto(search === "" ? "/" : `/?${search}`, {
      replaceState: !push,
      keepFocus: true,
      noScroll: true,
    });
  }
}
