import { test } from "node:test";
import { deepEqual, equal, rejects } from "node:assert/strict";
import { ApiError } from "./contracts.ts";
import { loadCatalog } from "./catalog.ts";

test("catalog transport handles plain and raw-gzip bytes without double decompression", async () => {
  const original = globalThis.fetch;
  const bytes = new TextEncoder().encode("test database transport");
  try {
    for (const gzip of [false, true]) {
      const body = gzip
        ? new Uint8Array(
            await new Response(
              new Blob([bytes]).stream().pipeThrough(new CompressionStream("gzip")),
            ).arrayBuffer(),
          )
        : bytes;
      globalThis.fetch = async (url) =>
        String(url).endsWith("/version") ? Response.json({ hash: "v1" }) : new Response(body);
      const loaded = await loadCatalog("test", new AbortController().signal);
      deepEqual(loaded.bytes, bytes);
      await loaded.commit();
    }
  } finally {
    globalThis.fetch = original;
  }
});

test("catalog transport rejects unauthorized, aborted and mixed-version downloads", async () => {
  const original = globalThis.fetch;
  try {
    globalThis.fetch = async () => new Response(null, { status: 401 });
    await rejects(
      loadCatalog("test", new AbortController().signal),
      (cause) => cause instanceof ApiError && cause.status === 401,
    );
    let versions = 0;
    globalThis.fetch = async (url) =>
      String(url).endsWith("/version")
        ? Response.json({ hash: `v${++versions}` })
        : new Response("bytes");
    await rejects(loadCatalog("test", new AbortController().signal), /changed during download/);
    equal(versions, 2);
    const controller = new AbortController();
    controller.abort();
    await rejects(loadCatalog("test", controller.signal));
  } finally {
    globalThis.fetch = original;
  }
});
