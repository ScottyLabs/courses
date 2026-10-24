// After a deploy, a tab opened before it still asks for the previous build's code files, which are gone: the server
// answers with index.html and the page's import fails. Reloading loads the new build. A guard in sessionStorage stops
// a reload loop when reloading doesn't help (the file really is missing, or the network is down).
// From Courseweave's lib/app-update.ts; React.lazy's wrapper becomes installUpdateReload() and handleStaleBuild().

const KEY = "cmucourses:reloaded-for-update";
/** Reload at most this often, in milliseconds. */
const GUARD_MS = 60_000;
let pending = false;

/** True for the errors browsers give when a dynamically imported file is missing or isn't JavaScript. */
export function isStaleBuildError(e: unknown): boolean {
  const message = e instanceof Error ? e.message : String(e);
  return /dynamically imported module|importing a module script failed|expected a javascript.*module script|unable to preload|failed to load module script|failed to fetch dynamically/iu.test(
    message,
  );
}

/** Reloads to pick up a new build, at most once a minute. False when it can't (it just did, or storage is off). */
export function reloadForUpdate(): boolean {
  if (pending) return true;
  try {
    if (Date.now() - Number(sessionStorage.getItem(KEY) ?? 0) < GUARD_MS) return false;
    sessionStorage.setItem(KEY, String(Date.now()));
  } catch {
    return false;
  }
  pending = true;
  location.reload();
  return true;
}

/**
 * For SvelteKit's `handleError` in hooks.client.ts: reloads when `error` is a stale-build import failure. True when
 * the page is reloading, so the error page needn't show.
 */
export function handleStaleBuild(error: unknown): boolean {
  return isStaleBuildError(error) && reloadForUpdate();
}

/**
 * Reloads when Vite fails to preload a chunk of an older build (its `vite:preloadError` event). Call once from the
 * root layout; returns a function that removes the listener.
 */
export function installUpdateReload(target: EventTarget = globalThis): () => void {
  const listener = (event: Event) => {
    // Vite rethrows the import error unless the event is cancelled; cancel it only when reloading.
    if (reloadForUpdate()) event.preventDefault();
  };
  target.addEventListener("vite:preloadError", listener);
  return () => {
    target.removeEventListener("vite:preloadError", listener);
  };
}
