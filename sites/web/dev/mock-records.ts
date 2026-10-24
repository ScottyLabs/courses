/* oxlint-disable eslint/max-classes-per-file -- one small class per kind of record, for one mock */
// The mock API's personal records (mock-server.ts): schedules, plans, the academic record, preferences, saved courses
// and a little of friends, in memory, with revisions and 409s like the web API (docs/src/api/web-api.md).

export type Json = Record<string, unknown>;

export const isObj = (v: unknown): v is Json =>
  typeof v === "object" && v !== null && !Array.isArray(v);

export function json(
  status: number,
  body: unknown,
  headers: Record<string, string> = {},
): Response {
  return new Response(body === undefined ? null : JSON.stringify(body), {
    status,
    headers: { "Content-Type": "application/json", "Cache-Control": "no-store", ...headers },
  });
}

export function noContent(): Response {
  return new Response(null, { status: 204 });
}

export function error(status: number, code: string, message: string, extra: Json = {}): Response {
  return json(status, { code, message, ...extra });
}

const RECORD_ID = /^[A-Za-z0-9_-]{1,64}$/u;
const TERM_ID = /^[SMF]\d{2}$/u;

export interface Revisioned extends Json {
  id: string;
  revision: number;
  createdAt: number;
  updatedAt: number;
}

/** A request to the records, already parsed. */
export interface RecordRequest {
  method: string;
  /** Path segments after /api/. */
  parts: string[];
  body: Json;
  url: URL;
}

/** Schedules or plans: records the student owns one by one. */
export class MockCollection {
  readonly records = new Map<string, Revisioned>();
  readonly #noun: "schedule" | "plan";
  readonly #now: () => number;

  constructor(noun: "schedule" | "plan", now: () => number) {
    this.#noun = noun;
    this.#now = now;
  }

  handle(req: RecordRequest): Response {
    const [, id, action] = req.parts;
    if (id === undefined) return req.method === "GET" ? this.#list(req.url) : this.#create(req);
    const current = this.records.get(id);
    if (current === undefined) return error(404, "not_found", `No such ${this.#noun}.`);
    if (action === "share" && this.#noun === "schedule") return this.#share(req, current);
    if (req.method === "DELETE") {
      this.records.delete(id);
      return noContent();
    }
    if (req.method === "PUT") return this.#update(req, current);
    return error(400, "bad_request", "Unsupported method.");
  }

  #list(url: URL): Response {
    const term = url.searchParams.get("term");
    return json(
      200,
      [...this.records.values()].filter((r) => term === null || r.termId === term),
    );
  }

  #invalid(body: Json): string | null {
    if (
      this.#noun === "schedule" &&
      (typeof body.termId !== "string" || !TERM_ID.test(body.termId))
    ) {
      return "termId is a term like S27.";
    }
    return typeof body.name === "string" && body.name.trim() !== "" ? null : "A name is required.";
  }

  #create(req: RecordRequest): Response {
    if (req.method !== "POST") return error(400, "bad_request", "Unsupported method.");
    const id = typeof req.body.id === "string" ? req.body.id : crypto.randomUUID();
    if (!RECORD_ID.test(id))
      return error(400, "bad_request", "Ids are 1 to 64 letters, digits, - or _.");
    const invalid = this.#invalid(req.body);
    if (invalid !== null) return error(400, "bad_request", invalid);
    const existing = this.records.get(id);
    if (existing !== undefined) {
      return error(409, "conflict", `A ${this.#noun} with this id exists.`, { current: existing });
    }
    const now = this.#now();
    const extra =
      this.#noun === "schedule"
        ? {
            visibility: req.body.visibility ?? "private",
            shareId: null,
            entries: req.body.entries ?? [],
          }
        : { programs: req.body.programs ?? [], terms: req.body.terms ?? [] };
    const created: Revisioned = {
      ...req.body,
      ...extra,
      id,
      revision: 1,
      createdAt: now,
      updatedAt: now,
    };
    this.records.set(id, created);
    return json(201, created);
  }

  #update(req: RecordRequest, current: Revisioned): Response {
    const { revision, ...fields } = req.body;
    if (typeof revision !== "number") return error(400, "bad_request", "A revision is required.");
    const invalid = this.#invalid(fields);
    if (invalid !== null) return error(400, "bad_request", invalid);
    if (revision !== current.revision) {
      return error(409, "conflict", "This changed since you loaded it.", { current });
    }
    const updated: Revisioned = {
      ...current,
      ...fields,
      id: current.id,
      revision: current.revision + 1,
      createdAt: current.createdAt,
      updatedAt: this.#now(),
    };
    this.records.set(current.id, updated);
    return json(200, updated);
  }

  #share(req: RecordRequest, current: Revisioned): Response {
    if (req.method === "DELETE") {
      this.records.set(current.id, { ...current, shareId: null });
      return noContent();
    }
    const shareId = typeof current.shareId === "string" ? current.shareId : crypto.randomUUID();
    this.records.set(current.id, { ...current, shareId });
    return json(200, { shareId, url: `${req.url.origin}/schedules/shared/${shareId}` });
  }

  /** A schedule shared by link, without its owner. */
  shared(token: string): Response {
    const s = [...this.records.values()].find((x) => x.shareId === token);
    if (s === undefined) return error(404, "not_found", "No schedule has this link.");
    return json(200, {
      termId: s.termId,
      name: s.name,
      entries: s.entries,
      updatedAt: s.updatedAt,
    });
  }
}

/** The academic record or preferences: one record per student, revision 0 until first saved. */
export class MockDocument {
  value: Json & { revision: number };
  readonly #now: () => number;

  constructor(initial: Json, now: () => number) {
    this.value = { ...initial, revision: 0, updatedAt: 0 };
    this.#now = now;
  }

  handle(req: RecordRequest): Response {
    if (req.method === "GET") return json(200, this.value);
    if (req.method !== "PUT") return error(400, "bad_request", "Unsupported method.");
    const { revision } = req.body;
    if (typeof revision !== "number") return error(400, "bad_request", "A revision is required.");
    if (revision !== this.value.revision) {
      return error(409, "conflict", "This changed since you loaded it.", { current: this.value });
    }
    this.value = {
      ...this.value,
      ...req.body,
      revision: this.value.revision + 1,
      updatedAt: this.#now(),
    };
    return json(200, this.value);
  }
}

/** Saved courses: no revisions, the last change to a course wins. */
export class MockSaved {
  readonly courses = new Map<string, number>();
  readonly #now: () => number;

  constructor(now: () => number) {
    this.#now = now;
  }

  handle(req: RecordRequest): Response {
    const [, courseId] = req.parts;
    if (courseId === undefined) {
      if (req.method !== "GET") return error(400, "bad_request", "Unsupported method.");
      return json(
        200,
        [...this.courses].map(([id, savedAt]) => ({ courseId: id, savedAt })),
      );
    }
    if (!/^\d{5}$/u.test(courseId)) return error(400, "bad_request", "Course ids are five digits.");
    if (req.method === "PUT") {
      if (!this.courses.has(courseId)) this.courses.set(courseId, this.#now());
      return noContent();
    }
    if (req.method === "DELETE") {
      this.courses.delete(courseId);
      return noContent();
    }
    return error(400, "bad_request", "Unsupported method.");
  }
}

/** Just enough of friends for the pages to render: nobody to befriend, sharing settings, invites. */
export class MockFriends {
  sharing = { shareCourses: false, shareSchedules: false };
  readonly #now: () => number;

  constructor(now: () => number) {
    this.#now = now;
  }

  handle(req: RecordRequest): Response {
    const [, id, action] = req.parts;
    if (id === undefined) {
      return req.method === "GET"
        ? json(200, [])
        : error(404, "not_found", "Nobody with that Andrew ID has signed in yet.");
    }
    if (id === "sharing") {
      if (req.method === "PUT") {
        this.sharing = {
          shareCourses: req.body.shareCourses === true,
          shareSchedules: req.body.shareSchedules === true,
        };
      }
      return json(200, this.sharing);
    }
    if (id === "courses") return json(200, {});
    if (id === "invites") return this.#invites(req, action);
    if (action === "schedules") return json(200, []);
    return error(404, "not_found", "No such friend.");
  }

  #invites(req: RecordRequest, token: string | undefined): Response {
    if (token === undefined && req.method === "POST") {
      const fresh = crypto.randomUUID();
      return json(200, {
        token: fresh,
        url: `${req.url.origin}/invite/${fresh}`,
        expiresAt: this.#now() + 7 * 86_400_000,
      });
    }
    if (req.method === "DELETE") return noContent();
    return error(404, "not_found", "This invite has expired or was used.");
  }
}
