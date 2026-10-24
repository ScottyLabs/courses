import { test } from "node:test";
import { deepEqual, equal, rejects } from "node:assert/strict";
import { api, ApiError, type User } from "./contracts.ts";
import { createDataClient } from "./data-client.ts";
import type { LocalCatalog } from "./data-source.ts";

const user: User = {
  subject: "first",
  name: "First",
  can_create: true,
  is_admin: false,
};
const result = (total: number) => ({
  items: [],
  total,
  page: 1,
  page_size: 12,
});
const deferred = <T>() => {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
};
function local(overrides: Partial<LocalCatalog> = {}): LocalCatalog {
  return {
    terms: async () => [],
    courses: async () => result(2),
    course: async () => undefined,
    instructors: async () => ({ items: [], total: 0 }),
    instructor: async () => undefined,
    geneds: async () => [],
    dispose: () => {},
    ...overrides,
  };
}
function ready(client: ReturnType<typeof createDataClient>) {
  return new Promise<void>((resolve) => {
    const stop = client.subscribeCatalog((state) => {
      if (state.status === "ready") {
        stop();
        resolve();
      }
    });
  });
}

test("API responds during warmup; ready catalog takes over; writes and private data stay remote", async () => {
  const loading = deferred<LocalCatalog>();
  let apiReads = 0, writes = 0, privateReads = 0;
  const client = createDataClient({
    ...api,
    me: async () => user,
    courses: async () => {
      apiReads++;
      return result(1);
    },
    deleteSchedule: async () => {
      writes++;
    },
    friends: async () => {
      privateReads++;
      return [];
    },
  }, () => loading.promise);
  const localReady = ready(client);
  await client.me();
  equal((await client.courses({ search: "15" })).total, 1);
  loading.resolve(local());
  await localReady;
  equal((await client.courses({ search: "15" })).total, 2);
  equal(apiReads, 1);
  await client.deleteSchedule("schedule");
  await client.friends();
  equal(writes, 1);
  equal(privateReads, 1);
});

test("unsupported local reads and local query failures fall back to the API", async () => {
  let disposed = 0;
  const client = createDataClient({
    ...api,
    me: async () => user,
    courses: async () => result(8),
    course: async () => {
      throw new ApiError(404, "Not found");
    },
  }, async () =>
    local({
      courses: async () => {
        throw new Error("Bad SQL");
      },
      dispose: () => {
        disposed++;
      },
    }));
  const localReady = ready(client);
  await client.me();
  await localReady;
  await rejects(
    client.course("missing"),
    (cause) => cause instanceof ApiError && cause.status === 404,
  );
  equal((await client.courses({})).total, 8);
  equal(disposed, 1);
});

test("failed local initialization preserves API use and can retry", async () => {
  let attempts = 0;
  const client = createDataClient({
    ...api,
    me: async () => user,
    courses: async () => result(1),
  }, async () => {
    if (++attempts === 1) throw new Error("Invalid database");
    return local();
  });
  const failed = new Promise<void>((resolve) =>
    client.subscribeCatalog((state) => {
      if (state.status === "unavailable") resolve();
    })
  );
  await client.me();
  await failed;
  equal((await client.courses({})).total, 1);
  const localReady = ready(client);
  await client.me();
  await localReady;
  equal((await client.courses({})).total, 2);
});

test("logout aborts warmup and disposes a late provider instead of activating it", async () => {
  const loading = deferred<LocalCatalog>();
  let signal!: AbortSignal, disposed = 0;
  const client = createDataClient({
    ...api,
    me: async () => user,
    courses: async () => result(1),
  }, (_, current) => {
    signal = current;
    return loading.promise;
  });
  await client.me();
  client.resetLocalCatalog();
  equal(signal.aborted, true);
  loading.resolve(local({
    dispose: () => {
      disposed++;
    },
  }));
  await loading.promise;
  await Promise.resolve();
  equal(disposed, 1);
  equal((await client.courses({})).total, 1);
});

test("account changes discard old in-flight local reads", async () => {
  let identity = user;
  const response = deferred<ReturnType<typeof result>>();
  const client = createDataClient(
    { ...api, me: async () => identity },
    async () => local({ courses: () => response.promise }),
  );
  const localReady = ready(client);
  await client.me();
  await localReady;
  const pending = client.courses({});
  identity = { ...user, subject: "second" };
  await client.me();
  response.resolve(result(99));
  await rejects(
    pending,
    (cause) => cause instanceof ApiError && cause.status === 401,
  );
});

test("a private API 401 disables local data; an initial anonymous identity does not invalidate public requests", async () => {
  let disposed = 0;
  const client = createDataClient({
    ...api,
    me: async () => user,
    courses: async () => result(1),
    friends: async () => {
      throw new ApiError(401, "Expired");
    },
  }, async () =>
    local({
      dispose: () => {
        disposed++;
      },
    }));
  const localReady = ready(client);
  await client.me();
  await localReady;
  await rejects(client.friends());
  equal(disposed, 1);
  equal((await client.courses({})).total, 1);
  const publicResult = deferred<ReturnType<typeof result>>();
  const anonymous = createDataClient({
    ...api,
    me: async () => {
      throw new ApiError(401, "Signed out");
    },
    courses: () => publicResult.promise,
  }, async () => {
    throw new Error("Must not load");
  });
  const pending = anonymous.courses({});
  await rejects(anonymous.me());
  publicResult.resolve(result(3));
  deepEqual(await pending, result(3));
});

test("anonymous users warm a public catalog without delaying or cancelling API reads", async () => {
  const loading = deferred<LocalCatalog>();
  let receivedUser: User | null | undefined;
  const client = createDataClient({ ...api,
    me: async () => { throw new ApiError(401,"Signed out"); },
    courses: async () => result(7),
  }, (user) => { receivedUser=user; return loading.promise; });
  const loaded=ready(client);
  await rejects(client.me());
  equal(receivedUser,null);
  equal((await client.courses({})).total,7);
  loading.resolve(local()); await loaded;
  equal((await client.courses({})).total,2);
  client.resetLocalCatalog();
});
