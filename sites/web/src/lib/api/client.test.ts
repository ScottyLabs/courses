/* oxlint-disable typescript/no-floating-promises, eslint/max-lines-per-function -- node:test awaits describe and it itself; a describe groups many cases */
import { describe, it } from "node:test";
import { expect } from "../testing.ts";
import { ApiClient, type FetchLike } from "./client.ts";
import { catalogPath, createApi, syllabusPath } from "./endpoints.ts";
import { ApiError, retryAfterSeconds } from "./errors.ts";

const reply =
  (status: number, body?: unknown, headers: Record<string, string> = {}): FetchLike =>
  () =>
    Promise.resolve(
      new Response(body === undefined ? null : JSON.stringify(body), {
        status,
        headers: body === undefined ? headers : { "Content-Type": "application/json", ...headers },
      }),
    );

const html: FetchLike = () =>
  Promise.resolve(
    new Response("<!doctype html>", { status: 200, headers: { "Content-Type": "text/html" } }),
  );

async function failure(promise: Promise<unknown>): Promise<ApiError> {
  try {
    await promise;
  } catch (error) {
    if (error instanceof ApiError) return error;
    throw error;
  }
  throw new Error("expected a failure");
}

describe("ApiClient", () => {
  it("sends JSON with the query and reads JSON back", async () => {
    const seen: { url: string; init?: RequestInit }[] = [];
    const client = new ApiClient({
      fetch: (url, init) => {
        seen.push({ url, init });
        return reply(200, { ok: true })(url, init);
      },
    });
    expect(
      await client.put("/api/x", { body: { a: 1 }, query: { term: "S27", skip: undefined } }),
    ).toEqual({ ok: true });
    expect(seen[0]?.url).toBe("/api/x?term=S27");
    expect(seen[0]?.init?.method).toBe("PUT");
    expect(seen[0]?.init?.body).toBe('{"a":1}');
    expect(seen[0]?.init?.credentials).toBe("same-origin");
  });

  it("returns nothing for 204", async () => {
    expect(await new ApiClient({ fetch: reply(204) }).delete("/api/x")).toBeUndefined();
  });

  it("turns error bodies into ApiError, with a 409's stored record", async () => {
    const current = { id: "s1", revision: 3 };
    const e = await failure(
      new ApiClient({ fetch: reply(409, { code: "conflict", message: "stale", current }) }).put(
        "/api/x",
      ),
    );
    expect(e.conflict).toBe(true);
    expect(e.code).toBe("conflict");
    expect(e.current).toEqual(current);
  });

  it("reads Retry-After on 429", async () => {
    const e = await failure(
      new ApiClient({
        fetch: reply(429, { code: "too_many", message: "slow" }, { "Retry-After": "120" }),
      }).get("/api/x"),
    );
    expect(e.rateLimited).toBe(true);
    expect(e.retryAfter).toBe(120);
    expect(retryAfterSeconds(null)).toBe(60);
    expect(
      retryAfterSeconds(new Date(Date.UTC(2030, 0, 1, 0, 2)).toUTCString(), Date.UTC(2030, 0, 1)),
    ).toBe(120);
  });

  it("reports 401s to the session and offline as its own code", async () => {
    const paths: string[] = [];
    const client = new ApiClient({ fetch: reply(401, { code: "auth", message: "Sign in" }) });
    client.onUnauthorized((path) => {
      paths.push(path);
    });
    expect((await failure(client.get("/api/record"))).signInRequired).toBe(true);
    expect(paths).toEqual(["/api/record"]);
    const offline = await failure(
      new ApiClient({ fetch: () => Promise.reject(new TypeError("Failed to fetch")) }).get(
        "/api/x",
      ),
    );
    expect(offline.offline).toBe(true);
    expect(offline.status).toBe(0);
  });

  it("treats HTML from a development server as the API being unavailable", async () => {
    expect((await failure(new ApiClient({ fetch: html }).get("/api/x"))).code).toBe("unavailable");
    const bare = await failure(new ApiClient({ fetch: reply(502) }).get("/api/x"));
    expect(bare.code).toBe("unavailable");
    expect(bare.status).toBe(502);
  });
});

describe("routes", () => {
  it("keeps the slashes of document names and keys", () => {
    expect(catalogPath("catalog/details/15")).toBe("/api/catalog/catalog/details/15");
    expect(catalogPath("term/S 27")).toBe("/api/catalog/term/S%2027");
    expect(syllabusPath("F25/MSC/21241 1.pdf", true)).toBe(
      "/api/syllabi/F25/MSC/21241%201.pdf?download=1",
    );
  });

  it("sends saved courses and friend requests where the API expects them", async () => {
    const seen: string[] = [];
    const api = createApi(
      new ApiClient({
        fetch: (url, init) => {
          seen.push(
            `${init?.method ?? "GET"} ${url} ${typeof init?.body === "string" ? init.body : ""}`.trim(),
          );
          return reply(204)(url, init);
        },
      }),
    );
    await api.saved.add("15122");
    await api.friends.request("scotty").catch(() => null);
    await api.schedules
      .update("a/b", { termId: "S27", name: "x", entries: [], visibility: "private", revision: 2 })
      .catch(() => null);
    expect(seen).toEqual([
      "PUT /api/saved/15122",
      'POST /api/friends {"andrewId":"scotty"}',
      'PUT /api/schedules/a%2Fb {"termId":"S27","name":"x","entries":[],"visibility":"private","revision":2}',
    ]);
  });
});
