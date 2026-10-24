/* oxlint-disable typescript/no-floating-promises, unicorn/no-useless-undefined -- node:test awaits describe and it itself; the fake API resolves 204 routes to undefined as the client does */
import { describe, it } from "node:test";
import { expect } from "../../testing.ts";
import { ApiError } from "../../api/errors.ts";
import type { Friend, Invite, Sharing } from "../../api/types.ts";
import { plainCell } from "../../data/cell.ts";
import {
  FriendsStore,
  friendlyError,
  isAndrewId,
  normalizeAndrewId,
  type FriendsApi,
} from "./friends.ts";
import { currentTerm, todayInPittsburgh } from "./term.ts";
import type { TermIndexEntry } from "../../core/types.ts";

// Made-up students.
function friend(id: string, name: string, status: Friend["status"]): Friend {
  return {
    id,
    name,
    andrewId: name.toLowerCase().replaceAll(" ", ""),
    status,
    sharesCourses: status === "accepted",
    sharesSchedules: false,
  };
}

class FakeFriends implements FriendsApi {
  friendList: Friend[] = [
    friend("1", "Bo Sample", "accepted"),
    friend("2", "Ada Example", "incoming"),
  ];
  stored: Sharing = { shareCourses: false, shareSchedules: false };
  fail: ApiError | null = null;
  calls: string[] = [];

  #check(call: string): void {
    this.calls.push(call);
    if (this.fail !== null) throw this.fail;
  }

  list = (): Promise<Friend[]> => {
    this.#check("list");
    return Promise.resolve(this.friendList);
  };

  request = (andrewId: string): Promise<Friend> => {
    this.#check(`request ${andrewId}`);
    return Promise.resolve(friend("3", "Cy Person", "outgoing"));
  };

  accept = (id: string): Promise<undefined> => {
    this.#check(`accept ${id}`);
    this.friendList = this.friendList.map((f) => (f.id === id ? { ...f, status: "accepted" } : f));
    return Promise.resolve(undefined);
  };

  remove = (id: string): Promise<undefined> => {
    this.#check(`remove ${id}`);
    return Promise.resolve(undefined);
  };

  invite = (): Promise<Invite> => {
    this.#check("invite");
    return Promise.resolve({
      token: "abc",
      url: "https://example.test/invite/abc",
      expiresAt: Date.now() + 1000,
    });
  };

  revokeInvite = (token: string): Promise<undefined> => {
    this.#check(`revoke ${token}`);
    return Promise.resolve(undefined);
  };

  sharing = (): Promise<Sharing> => {
    this.#check("sharing");
    return Promise.resolve(this.stored);
  };

  setSharing = (body: Sharing): Promise<Sharing> => {
    this.#check("setSharing");
    this.stored = body;
    return Promise.resolve(body);
  };
}

describe("Andrew IDs", () => {
  it("drops the email domain, spaces and case", () => {
    expect(normalizeAndrewId("  JDoe@andrew.cmu.edu ")).toBe("jdoe");
    expect(isAndrewId("jdoe")).toBe(true);
    expect(isAndrewId("j doe")).toBe(false);
    expect(isAndrewId("")).toBe(false);
  });
});

describe("friendlyError", () => {
  it("passes the server's explanation through and words the rest", () => {
    expect(friendlyError(new ApiError(400, "bad_request", "You can't add yourself."), "x")).toBe(
      "You can't add yourself.",
    );
    expect(friendlyError(new ApiError(0, "network", "offline"), "x")).toContain("offline");
    expect(friendlyError(new ApiError(500, "unavailable", "boom"), "Try again.")).toBe(
      "Try again.",
    );
    expect(friendlyError(new Error("?"), "Try again.")).toBe("Try again.");
  });
});

describe("FriendsStore", () => {
  it("loads friends and sharing and splits them by status", async () => {
    const store = new FriendsStore(new FakeFriends(), plainCell);
    await store.load();
    expect(store.status).toBe("ready");
    expect(store.accepted.map((f) => f.name)).toEqual(["Bo Sample"]);
    expect(store.incoming.map((f) => f.name)).toEqual(["Ada Example"]);
    expect(store.sharing).toEqual({ shareCourses: false, shareSchedules: false });
  });

  it("explains a failed load", async () => {
    const api = new FakeFriends();
    api.fail = new ApiError(0, "network", "offline");
    const store = new FriendsStore(api, plainCell);
    await store.load();
    expect(store.status).toBe("error");
    expect(store.error).toContain("offline");
  });

  it("sends requests by normalized Andrew ID and lists them", async () => {
    const api = new FakeFriends();
    const store = new FriendsStore(api, plainCell);
    await store.load();
    await store.request(" CyPerson@andrew.cmu.edu");
    expect(api.calls).toContain("request cyperson");
    expect(store.outgoing.map((f) => f.name)).toEqual(["Cy Person"]);
  });
});

describe("FriendsStore changes", () => {
  it("accepts and removes", async () => {
    const api = new FakeFriends();
    const store = new FriendsStore(api, plainCell);
    await store.load();
    await store.accept("2");
    expect(store.incoming).toHaveLength(0);
    expect(store.accepted).toHaveLength(2);
    await store.remove("1");
    expect(store.all.map((f) => f.id)).toEqual(["2"]);
  });

  it("treats removing a friendship that's already gone as done", async () => {
    const api = new FakeFriends();
    const store = new FriendsStore(api, plainCell);
    await store.load();
    api.fail = new ApiError(404, "not_found", "No such friend.");
    await store.remove("1");
    expect(store.all.map((f) => f.id)).toEqual(["2"]);
  });

  it("keeps the invite made here until it's revoked", async () => {
    const api = new FakeFriends();
    const store = new FriendsStore(api, plainCell);
    expect(store.invite).toBeNull();
    await store.createInvite();
    expect(store.invite?.token).toBe("abc");
    await store.revokeInvite();
    expect(api.calls).toContain("revoke abc");
    expect(store.invite).toBeNull();
  });
});

describe("FriendsStore sharing", () => {
  it("puts sharing back when the server refuses the change", async () => {
    const api = new FakeFriends();
    const store = new FriendsStore(api, plainCell);
    await store.load();
    await store.setSharing({ shareCourses: true });
    expect(store.sharing).toEqual({ shareCourses: true, shareSchedules: false });
    api.fail = new ApiError(429, "too_many", "slow down");
    await expect(store.setSharing({ shareSchedules: true })).rejects.toThrow("slow down");
    expect(store.sharing).toEqual({ shareCourses: true, shareSchedules: false });
  });

  it("forgets everything on reset", async () => {
    const store = new FriendsStore(new FakeFriends(), plainCell);
    await store.load();
    await store.createInvite();
    store.reset();
    expect(store.all).toHaveLength(0);
    expect(store.invite).toBeNull();
    expect(store.status).toBe("idle");
  });
});

function term(id: string, classesBegin: string, classesEnd: string): TermIndexEntry {
  return {
    id,
    name: id,
    season: "fall",
    year: 2026,
    minis: [],
    classesBegin,
    classesEnd,
    noClasses: [],
    courseCount: 0,
    sectionCount: 0,
  };
}

describe("currentTerm", () => {
  const terms = [
    term("S27", "2027-01-19", "2027-04-30"),
    term("F26", "2026-08-24", "2026-12-04"),
    term("M26", "2026-05-18", "2026-08-07"),
  ];

  it("picks the semester in session, else the next one, else the newest", () => {
    expect(currentTerm(terms, "2026-10-24")).toBe("F26");
    expect(currentTerm(terms, "2026-12-20")).toBe("S27");
    expect(currentTerm(terms, "2027-06-01")).toBe("S27");
    expect(currentTerm([], "2027-06-01")).toBeNull();
  });

  it("formats today in Pittsburgh", () => {
    expect(todayInPittsburgh(new Date("2026-10-25T02:00:00Z"))).toBe("2026-10-24");
  });
});
