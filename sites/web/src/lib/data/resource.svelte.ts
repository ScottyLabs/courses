// A loader's result as reactive state, for components: `const term = resource(() => catalog.term(termId))`, then
// `term.status` and `term.value` in the markup. It loads again whenever something `load` reads changes (the term id,
// or the catalog's session generation). Call it while a component initializes, like any $effect.
import { settle } from "./catalog.ts";

export type ResourceStatus = "idle" | "loading" | "ready" | "signin" | "forbidden" | "error";

export interface Resource<T> {
  /**
   * idle: `load` returned null (nothing to load); signin: a member-only document while signed out; forbidden: the
   * same, signed in with an account that isn't a CMU one.
   */
  readonly status: ResourceStatus;
  /** The latest loaded value. Kept while the next one loads, so pages don't flash empty. */
  readonly value: T | undefined;
  /** What went wrong, when status is "error". */
  readonly error: string | null;
}

export function resource<T>(load: () => Promise<T> | null | undefined): Resource<T> {
  let status = $state<ResourceStatus>("loading");
  let value = $state.raw<T>();
  let error = $state<string | null>(null);

  $effect(() => {
    const promise = load();
    let current = true;
    if (promise === null || promise === undefined) status = "idle";
    else {
      status = "loading";
      void settle(promise).then((result) => {
        if (!current) return;
        if (result.status === "ready") {
          value = result.value;
          error = null;
        } else if (result.status === "error") error = result.error;
        status = result.status;
      });
    }
    return () => {
      current = false;
    };
  });

  return {
    get status() {
      return status;
    },
    get value() {
      return value;
    },
    get error() {
      return error;
    },
  };
}
