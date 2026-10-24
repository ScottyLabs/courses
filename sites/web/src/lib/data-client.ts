import { type api as Api, ApiError } from "./contracts";
import type { CatalogState, LocalCatalog, LocalCatalogFactory } from "./data-source";

type RemoteApi = typeof Api;
type CatalogMethod = "terms" | "courses" | "course" | "instructors" | "instructor" | "geneds";
const SESSION_CHECK_MS = 60_000;

/** UI-facing repository: API during warmup; local catalog reads after readiness. */
export function createDataClient(remote: RemoteApi, loadLocal: LocalCatalogFactory) {
  let local: LocalCatalog | undefined;
  let controller: AbortController | undefined;
  let generation = 0;
  let scope = "";
  let state: CatalogState = { status: "idle" };
  let checkedAt = 0;
  let identityRequest: ReturnType<RemoteApi["me"]> | undefined;
  const listeners = new Set<(state: CatalogState) => void>();

  function publish(next: CatalogState) {
    state = next;
    for (const listener of listeners) listener({ ...state });
  }

  function reset() {
    generation++;
    controller?.abort();
    controller = undefined;
    local?.dispose();
    local = undefined;
    scope = "";
    checkedAt = 0;
    publish({ status: "idle" });
  }

  function warmPublic() {
    if (scope === "public" && state.status !== "unavailable") return;
    if (scope) reset();
    scope = "public";
    checkedAt = Date.now();
    controller = new AbortController();
    const signal = controller.signal;
    const started = generation;
    publish({ status: "loading" });
    void loadLocal(null, signal)
      .then((loaded) => {
        if (signal.aborted || started !== generation) {
          loaded.dispose();
          return;
        }
        local = loaded;
        publish({ status: "ready" });
      })
      .catch((cause) => {
        if (!signal.aborted && started === generation) {
          publish({ status: "unavailable", error: String(cause) });
        }
      });
  }

  async function me() {
    if (identityRequest) return identityRequest;
    const started = generation;
    identityRequest = remote
      .me()
      .then((user) => {
        if (started !== generation) {
          throw new ApiError(401, "Session changed. Please try again.");
        }
        checkedAt = Date.now();
        const nextScope = JSON.stringify([user.subject, user.can_create, user.is_admin]);
        if (scope !== nextScope || state.status === "unavailable") {
          if (scope && scope !== nextScope) reset();
          checkedAt = Date.now();
          scope = nextScope;
          controller = new AbortController();
          const signal = controller.signal;
          const loadingGeneration = generation;
          publish({ status: "loading" });
          // Loading never delays the identity request or the first catalog query.
          void Promise.resolve()
            .then(() => loadLocal(user, signal))
            .then((loaded) => {
              if (signal.aborted || loadingGeneration !== generation) {
                loaded.dispose();
                return;
              }
              local = loaded;
              publish({ status: "ready" });
            })
            .catch((cause) => {
              if (signal.aborted || loadingGeneration !== generation) return;
              publish({
                status: "unavailable",
                error: cause instanceof Error ? cause.message : "Local catalog unavailable",
              });
            });
        }
        return user;
      })
      .catch((cause) => {
        if (started === generation) {
          if (cause instanceof ApiError && cause.status === 401) warmPublic();
          else if (scope) reset();
        }
        throw cause;
      })
      .finally(() => {
        identityRequest = undefined;
      });
    return identityRequest;
  }

  async function read<K extends CatalogMethod>(
    method: K,
    args: Parameters<RemoteApi[K]>,
  ): Promise<Awaited<ReturnType<RemoteApi[K]>>> {
    if (local) {
      // Recheck the session before using protected cached data after the TTL.
      if (Date.now() - checkedAt >= SESSION_CHECK_MS) {
        try {
          await me();
        } catch (cause) {
          if (!(cause instanceof ApiError && cause.status === 401 && scope === "public"))
            throw cause;
          checkedAt = Date.now();
        }
      }
      const provider = local;
      const started = generation;
      if (provider) {
        try {
          const result = await (provider[method] as (...values: typeof args) => Promise<unknown>)(
            ...args,
          );
          if (started !== generation) {
            throw new ApiError(401, "Session changed. Please try again.");
          }
          if (result !== undefined) {
            return result as Awaited<ReturnType<RemoteApi[K]>>;
          }
        } catch (cause) {
          if (started !== generation) throw cause;
          // A broken local adapter should not make the application unusable.
          provider.dispose();
          local = undefined;
          publish({
            status: "unavailable",
            error: cause instanceof Error ? cause.message : "Local query failed",
          });
        }
      }
    }
    const started = generation;
    try {
      const result = await (remote[method] as (...values: typeof args) => Promise<unknown>)(
        ...args,
      );
      if (started !== generation) {
        throw new ApiError(401, "Session changed. Please try again.");
      }
      return result as Awaited<ReturnType<RemoteApi[K]>>;
    } catch (cause) {
      if (started === generation && cause instanceof ApiError && [401, 403].includes(cause.status))
        reset();
      throw cause;
    }
  }

  const server = Object.fromEntries(
    Object.entries(remote).map(([name, method]) => [
      name,
      async (...args: unknown[]) => {
        const started = generation;
        try {
          return await (method as (...values: unknown[]) => Promise<unknown>)(...args);
        } catch (cause) {
          if (
            started === generation &&
            cause instanceof ApiError &&
            [401, 403].includes(cause.status)
          )
            reset();
          throw cause;
        }
      },
    ]),
  ) as RemoteApi;

  return {
    ...server, // Personal data and every mutation always use the API.
    me,
    terms: () => read("terms", []),
    courses: (...args: Parameters<RemoteApi["courses"]>) => read("courses", args),
    course: (...args: Parameters<RemoteApi["course"]>) => read("course", args),
    instructors: (...args: Parameters<RemoteApi["instructors"]>) => read("instructors", args),
    instructor: (...args: Parameters<RemoteApi["instructor"]>) => read("instructor", args),
    geneds: () => read("geneds", []),
    resetLocalCatalog: reset,
    subscribeCatalog(listener: (state: CatalogState) => void) {
      listeners.add(listener);
      listener({ ...state });
      return () => {
        listeners.delete(listener);
      };
    },
  };
}
