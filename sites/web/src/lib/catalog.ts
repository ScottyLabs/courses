import { ApiError } from "./contracts";

const PREFIX = "catalog-sqlite-v1-";
const MAX_BYTES = 128 * 1024 * 1024;

async function version(signal: AbortSignal): Promise<string> {
  const response = await fetch("/api/catalog/version", {
    credentials: "same-origin",
    cache: "no-store",
    signal,
  });
  if (!response.ok) {
    throw new ApiError(response.status, "Catalog version unavailable");
  }
  const value = await response.json();
  if (typeof value?.hash !== "string" || !value.hash) {
    throw new Error("Invalid catalog version");
  }
  return value.hash;
}

async function digest(value: string) {
  const bytes = await crypto.subtle.digest(
    "SHA-256",
    new TextEncoder().encode(value),
  );
  return Array.from(
    new Uint8Array(bytes),
    (byte) => byte.toString(16).padStart(2, "0"),
  ).join("");
}

async function directory() {
  try {
    return await navigator.storage?.getDirectory();
  } catch {
    return undefined;
  } // Unsupported/private storage: use memory instead.
}

async function remove(
  root: FileSystemDirectoryHandle | undefined,
  name: string,
) {
  try {
    await root?.removeEntry(name);
  } catch { /* best effort */ }
}

async function readLimited(
  stream: ReadableStream<Uint8Array>,
  signal: AbortSignal,
) {
  const reader = stream.getReader();
  const chunks: Uint8Array<ArrayBuffer>[] = [];
  let size = 0;
  try {
    while (true) {
      signal.throwIfAborted();
      const { value, done } = await reader.read();
      if (done) break;
      size += value.byteLength;
      if (size > MAX_BYTES) {
        throw new Error("Catalog exceeds the browser size limit");
      }
      chunks.push(new Uint8Array(value));
    }
  } finally {
    await reader.cancel().catch(() => {});
  }
  return new Uint8Array(await new Blob(chunks).arrayBuffer());
}

/** A cache hit still verifies the server's current version and authenticated access. */
export async function loadCatalog(scope: string, signal: AbortSignal) {
  const hash = await version(signal);
  const prefix = `${PREFIX}${await digest(scope)}-`;
  const filename = `${prefix}${await digest(hash)}.sqlite`;
  const root = await directory();
  signal.throwIfAborted();
  let bytes: Uint8Array<ArrayBuffer> | undefined;
  try {
    const file = await (await root?.getFileHandle(filename))?.getFile();
    if (file && file.size <= MAX_BYTES) {
      bytes = new Uint8Array(await file.arrayBuffer());
    }
  } catch { /* miss or storage unavailable */ }
  const fromCache = !!bytes;
  if (!bytes) {
    const response = await fetch("/api/catalog/binary", {
      credentials: "same-origin",
      cache: "no-store",
      signal,
    });
    if (!response.ok) {
      throw new ApiError(response.status, "Catalog download unavailable");
    }
    if (!response.body) throw new Error("Empty catalog download");
    bytes = await readLimited(response.body, signal);
    // Fetch decodes HTTP Content-Encoding; only decompress bytes still bearing gzip magic.
    if (bytes[0] === 0x1f && bytes[1] === 0x8b) {
      bytes = await readLimited(
        new Blob([bytes]).stream().pipeThrough(new DecompressionStream("gzip")),
        signal,
      );
    }
    if (await version(signal) !== hash) {
      throw new Error("Catalog changed during download");
    }
  }
  signal.throwIfAborted();
  if (/^[a-f0-9]{64}$/.test(hash)) {
    const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", bytes));
    const actual = Array.from(digest, (byte) => byte.toString(16).padStart(2, "0")).join("");
    if (actual !== hash) {
      await remove(root, filename);
      throw new Error("Catalog checksum mismatch; retry after publication completes");
    }
  }
  const validatedBytes = bytes;
  return {
    bytes,
    invalidate: () => remove(root, filename),
    // Call only after SQLite has opened the file and validated the schema.
    async commit() {
      if (!root || fromCache) return;
      signal.throwIfAborted();
      try {
        const writable =
          await (await root.getFileHandle(filename, { create: true }))
            .createWritable();
        try {
          await writable.write(validatedBytes);
          signal.throwIfAborted();
          await writable.close();
        } catch (cause) {
          await writable.abort().catch(() => {});
          throw cause;
        }
        for await (const name of root.keys()) {
          if (name.startsWith(prefix) && name !== filename) {
            await remove(root, name);
          }
        }
      } catch { /* Quota/cache failure must not prevent local queries. */ }
    },
  };
}

export async function clearCatalogCache() {
  const root = await directory();
  if (!root) return;
  try {
    for await (const name of root.keys()) {
      if (name.startsWith(PREFIX) || name.startsWith("catalog-v1-")) await remove(root, name);
    }
  } catch { /* best effort */ }
}
