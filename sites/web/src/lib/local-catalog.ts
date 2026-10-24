import { loadCatalog } from "./catalog";
import { createSQLiteCatalog } from "./sqlite-catalog";
import type { User } from "./contracts";
import type { LocalCatalog } from "./data-source";

export async function openLocalCatalog(
  user: User | null,
  signal: AbortSignal,
): Promise<LocalCatalog> {
  const scope = user ? JSON.stringify([user.subject, user.can_create, user.is_admin]) : "public";
  const download = await loadCatalog(scope, signal);
  signal.throwIfAborted();
  const worker = new Worker(new URL("./sqlite.worker.ts", import.meta.url), {
    type: "module",
  });
  let id = 0;
  let disposed = false;
  const pending = new Map<
    number,
    { resolve: (value: any) => void; reject: (error: Error) => void }
  >();
  const dispose = () => {
    if (disposed) return;
    disposed = true;
    worker.terminate();
    for (const request of pending.values()) {
      request.reject(new Error("Catalog closed"));
    }
    pending.clear();
    signal.removeEventListener("abort", dispose);
  };
  worker.onmessage = ({ data }) => {
    const request = pending.get(data.id);
    pending.delete(data.id);
    if (data.error) request?.reject(new Error(data.error));
    else request?.resolve(data.result);
  };
  worker.onerror = dispose;
  const send = (message: object): Promise<any> =>
    new Promise((resolve, reject) => {
      if (disposed) {
        reject(new Error("Catalog closed"));
        return;
      }
      pending.set(++id, { resolve, reject });
      worker.postMessage({ ...message, id });
    });
  signal.addEventListener("abort", dispose, { once: true });
  try {
    signal.throwIfAborted();
    await send({ bytes: download.bytes });
    const local = await createSQLiteCatalog(
      {
        query: (sql, params) => send({ sql, params }),
      },
      dispose,
    );
    await download.commit();
    signal.throwIfAborted();
    // Public files have no FCE values, but reject protected filters via the API.
    if (!user) {
      const courses = local.courses;
      local.courses = (query) =>
        query.min_fce !== undefined && query.min_fce !== ""
          ? Promise.resolve(undefined)
          : courses(query);
    }
    return local;
  } catch (cause) {
    dispose();
    await download.invalidate();
    throw cause;
  }
}
