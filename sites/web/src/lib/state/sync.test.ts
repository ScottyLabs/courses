/* oxlint-disable typescript/no-floating-promises, eslint/max-lines-per-function, typescript/no-unsafe-type-assertion -- node:test awaits describe and it itself; a describe groups many cases; fixtures are cast to their shapes */
import { describe, it } from "node:test";
import { ApiError } from "../api/errors.ts";
import { expect } from "../testing.ts";
import { RecordSync, worstStatus, type RecordSyncOptions, type Stored } from "./sync.ts";
import { FakeEnv } from "./test-env.ts";

interface Doc {
  items: string[];
}

/** A server holding one Doc, with a log of writes. */
function server(initial?: Stored<Doc>) {
  const state = {
    stored: initial ?? { value: { items: [] }, revision: 1 },
    writes: [] as { value: Doc; revision: number | null }[],
    fail: [] as ApiError[],
  };
  const save = (value: Doc, revision: number | null): Promise<Stored<Doc>> => {
    state.writes.push({ value, revision });
    const failure = state.fail.shift();
    if (failure !== undefined) return Promise.reject(failure);
    if (revision !== null && revision !== state.stored.revision) {
      const current = { ...state.stored.value, revision: state.stored.revision };
      return Promise.reject(new ApiError(409, "conflict", "stale", { current }));
    }
    state.stored = { value: { items: [...value.items] }, revision: state.stored.revision + 1 };
    return Promise.resolve(state.stored);
  };
  return { state, save };
}

const conflict = () =>
  new ApiError(409, "conflict", "stale", { current: { items: [], revision: 9 } });

function noop(): void {
  // Nothing to hear.
}

function sync(
  env: FakeEnv,
  save: RecordSyncOptions<Doc>["save"],
  init: Partial<{ revision: number; editsKnown: boolean; value: Doc; unsaved: boolean }> = {},
) {
  const seen: Doc[] = [];
  const s = new RecordSync<Doc>(
    {
      env,
      debounceMs: 1000,
      save,
      parse: (raw) => {
        const r = raw as { items?: unknown; revision?: unknown };
        return Array.isArray(r.items) && typeof r.revision === "number"
          ? { value: { items: r.items as string[] }, revision: r.revision }
          : null;
      },
      // Courseweave-style union, for edits that aren't known.
      merge: (local, remote) => ({ items: [...new Set([...local.items, ...remote.items])] }),
      onValue: (v) => {
        seen.push(v);
      },
      onStatus: noop,
      recreateWhenMissing: true,
    },
    {
      value: init.value ?? { items: [] },
      revision: init.revision ?? 1,
      unsaved: init.unsaved ?? false,
      editsKnown: init.editsKnown ?? true,
    },
  );
  return { s, seen };
}

const add =
  (item: string) =>
  (d: Doc): Doc =>
    d.items.includes(item) ? d : { items: [...d.items, item] };

describe("RecordSync", () => {
  it("debounces a burst of edits into one write", async () => {
    const env = new FakeEnv();
    const { state, save } = server();
    const { s } = sync(env, save);
    s.edit(add("a"));
    await env.advance(500);
    s.edit(add("b"));
    await env.advance(500);
    s.edit(add("c"));
    expect(s.status).toBe("saving");
    expect(state.writes).toHaveLength(0);
    await env.advance(1000);
    expect(state.writes).toEqual([{ value: { items: ["a", "b", "c"] }, revision: 1 }]);
    expect(s.status).toBe("saved");
    expect(s.revision).toBe(2);
    expect(s.unsaved).toBe(false);
  });

  it("replays this device's edits onto the stored record after a conflict", async () => {
    const env = new FakeEnv();
    const { state, save } = server();
    const { s } = sync(env, save);
    // Another device writes first.
    state.stored = { value: { items: ["theirs"] }, revision: 2 };
    s.edit(add("mine"));
    await env.advance(1000);
    expect(state.writes.map((w) => w.revision)).toEqual([1, 2]);
    expect(state.stored).toEqual({ value: { items: ["theirs", "mine"] }, revision: 3 });
    expect(s.value).toEqual({ items: ["theirs", "mine"] });
    expect(s.status).toBe("saved");
  });

  it("merges by Courseweave's rules when the edits aren't known", async () => {
    const env = new FakeEnv();
    const { state, save } = server({ value: { items: ["theirs"] }, revision: 5 });
    const { s } = sync(env, save, {
      revision: 3,
      value: { items: ["draft"] },
      unsaved: true,
      editsKnown: false,
    });
    await env.advance(1000);
    expect(state.stored).toEqual({ value: { items: ["draft", "theirs"] }, revision: 6 });
    expect(s.value.items).toEqual(["draft", "theirs"]);
  });

  it("keeps edits made while a write is out and saves them next", async () => {
    const env = new FakeEnv();
    const { state, save } = server();
    let release: () => void = noop;
    const gate = new Promise<void>((resolve) => {
      release = resolve;
    });
    const slowSave: typeof save = async (value, revision) => {
      if (state.writes.length === 0) await gate;
      return save(value, revision);
    };
    const { s } = sync(env, slowSave);
    s.edit(add("a"));
    await env.advance(1000);
    s.edit(add("b"));
    release();
    await env.advance(1000);
    expect(state.writes.map((w) => w.value.items)).toEqual([["a"], ["a", "b"]]);
    expect(s.value.items).toEqual(["a", "b"]);
    expect(s.status).toBe("saved");
  });

  it("waits offline and saves when retried", async () => {
    const env = new FakeEnv();
    const { state, save } = server();
    const { s } = sync(env, save);
    state.fail.push(new ApiError(0, "network", "offline"));
    s.edit(add("a"));
    await env.advance(1000);
    expect(s.status).toBe("offline");
    expect(s.unsaved).toBe(true);
    s.retry();
    await env.advance(0);
    expect(s.status).toBe("saved");
    expect(state.stored.value.items).toEqual(["a"]);
  });

  it("does nothing while the browser says it's offline", async () => {
    const env = new FakeEnv();
    env.isOnline = false;
    const { state, save } = server();
    const { s } = sync(env, save);
    s.edit(add("a"));
    await env.advance(5000);
    expect(state.writes).toHaveLength(0);
    expect(s.status).toBe("offline");
  });

  it("honors Retry-After on 429", async () => {
    const env = new FakeEnv();
    const { state, save } = server();
    const { s } = sync(env, save);
    state.fail.push(new ApiError(429, "too_many", "slow down", { retryAfter: 120 }));
    s.edit(add("a"));
    await env.advance(1000);
    expect(s.status).toBe("error");
    expect(s.error).toContain("in 2 minutes");
    await env.advance(119_000);
    expect(state.writes).toHaveLength(1);
    await env.advance(1000);
    expect(state.writes).toHaveLength(2);
    expect(s.status).toBe("saved");
  });

  it("backs off after server errors", async () => {
    const env = new FakeEnv();
    const { state, save } = server();
    const { s } = sync(env, save);
    state.fail.push(
      new ApiError(500, "unavailable", "boom"),
      new ApiError(500, "unavailable", "boom"),
    );
    s.edit(add("a"));
    await env.advance(1000);
    await env.advance(30_000);
    expect(state.writes).toHaveLength(2);
    await env.advance(59_000);
    expect(state.writes).toHaveLength(2);
    await env.advance(1000);
    expect(state.writes).toHaveLength(3);
    expect(s.status).toBe("saved");
  });

  it("recreates a record deleted elsewhere", async () => {
    const env = new FakeEnv();
    const { state, save } = server();
    const { s } = sync(env, save);
    state.fail.push(new ApiError(404, "not_found", "gone"));
    s.edit(add("a"));
    await env.advance(1000);
    expect(state.writes.map((w) => w.revision)).toEqual([1, null]);
  });

  it("gives up after repeated conflicts", async () => {
    const env = new FakeEnv();
    const { state, save } = server();
    const { s } = sync(env, save);
    state.fail.push(conflict(), conflict(), conflict(), conflict());
    s.edit(add("a"));
    await env.advance(1000);
    expect(s.status).toBe("error");
    expect(state.writes).toHaveLength(4);
  });
});

describe("worstStatus", () => {
  it("shows the most pressing status", () => {
    expect(worstStatus([])).toBe("saved");
    expect(worstStatus(["saved", "saving"])).toBe("saving");
    expect(worstStatus(["saving", "offline"])).toBe("offline");
    expect(worstStatus(["offline", "error", "saved"])).toBe("error");
  });
});
