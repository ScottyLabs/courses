import type { components } from "$lib/api";

type CatalogVersion = components["schemas"]["CatalogVersion"];

function isCatalogVersion(value: unknown): value is CatalogVersion {
  return (
    typeof value === "object" && value !== null && "hash" in value && typeof value.hash === "string"
  );
}

const CATALOG_PREFIX = "catalog-";

export type CatalogLoad = {
  bytes: Uint8Array;
  fromCache: boolean;
  versionMs: number;
  fetchMs: number;
  decompressMs: number;
  hash: string;
};

/**
 * Load the catalog binary, preferring an OPFS-cached copy keyed by the
 * server's content hash. On a hash miss, downloads `/api/catalog/binary`,
 * persists the gzipped bytes to OPFS, and prunes stale catalogs.
 *
 * Returns the decompressed bytes ready to hand to `WasmIndex`.
 */
export async function loadCatalog(): Promise<CatalogLoad> {
  const t0 = performance.now();
  const version: unknown = await (await fetch("/api/catalog/version")).json();
  if (!isCatalogVersion(version)) {
    throw new Error("invalid /api/catalog/version response");
  }
  const t1 = performance.now();

  const root = await navigator.storage.getDirectory();
  const filename = `${CATALOG_PREFIX}${version.hash}.bin`;

  let gzipped = await readOpfs(root, filename);
  let fromCache = gzipped !== null;

  if (!gzipped) {
    const response = await fetch("/api/catalog/binary");
    gzipped = new Uint8Array(await response.arrayBuffer());
    await writeOpfs(root, filename, gzipped);
    await pruneStale(root, filename);
  }
  const t2 = performance.now();

  const decompressed = new Uint8Array(
    await new Response(
      new Blob([gzipped]).stream().pipeThrough(new DecompressionStream("gzip")),
    ).arrayBuffer(),
  );
  const t3 = performance.now();

  return {
    bytes: decompressed,
    fromCache,
    versionMs: t1 - t0,
    fetchMs: t2 - t1,
    decompressMs: t3 - t2,
    hash: version.hash,
  };
}

async function readOpfs(
  root: FileSystemDirectoryHandle,
  name: string,
): Promise<Uint8Array<ArrayBuffer> | null> {
  try {
    const handle = await root.getFileHandle(name);
    const file = await handle.getFile();
    return new Uint8Array(await file.arrayBuffer());
  } catch (e) {
    if (e instanceof DOMException && e.name === "NotFoundError") return null;
    throw e;
  }
}

async function writeOpfs(
  root: FileSystemDirectoryHandle,
  name: string,
  bytes: Uint8Array<ArrayBuffer>,
): Promise<void> {
  const handle = await root.getFileHandle(name, { create: true });
  // FileSystemWritableFileStream is available on every browser that ships
  // OPFS (Chromium, Firefox, Safari 15.2+).
  const stream = await handle.createWritable();
  await stream.write(bytes);
  await stream.close();
}

async function pruneStale(root: FileSystemDirectoryHandle, keep: string): Promise<void> {
  for await (const [name] of root.entries()) {
    if (name.startsWith(CATALOG_PREFIX) && name !== keep) {
      try {
        await root.removeEntry(name);
      } catch {
        // best effort; another tab may have raced us
      }
    }
  }
}
