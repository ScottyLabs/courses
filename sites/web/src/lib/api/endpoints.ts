// One function per web API route (openapi.json; docs/src/api/web-api.md). Catalog documents are loaded through
// lib/data, which caches them; everything else goes through here.
import type { ApiClient } from "./client.ts";
import type {
  AcademicRecord,
  FinalExam,
  Friend,
  FriendSchedule,
  FriendsTaking,
  Invite,
  InviteInfo,
  Me,
  NewPlan,
  NewSchedule,
  PlanRecord,
  PlanUpdate,
  Preferences,
  PreferencesUpdate,
  RecordUpdate,
  SavedCourse,
  ScheduleRecord,
  ScheduleUpdate,
  ShareLink,
  SharedSchedule,
  Sharing,
} from "./types.ts";

const seg = (s: string): string => encodeURIComponent(s);

/** A path with each segment of `name` encoded and its slashes kept ("term/S27"). */
function segments(name: string): string {
  return name
    .split("/")
    .map((s) => seg(s))
    .join("/");
}

/** A catalog document's path. */
export function catalogPath(name: string): string {
  return `/api/catalog/${segments(name)}`;
}

/** Where a hosted syllabus file opens (PDFs show in the browser) or downloads. */
export function syllabusPath(key: string, download = false): string {
  return `/api/syllabi/${segments(key)}${download ? "?download=1" : ""}`;
}

function schedulesApi(client: ApiClient) {
  return {
    list: (term?: string) => client.get<ScheduleRecord[]>("/api/schedules", { query: { term } }),
    create: (body: NewSchedule) => client.post<ScheduleRecord>("/api/schedules", { body }),
    /** Replaces a schedule. A stale revision throws ApiError 409 with the stored schedule as `current`. */
    update: (id: string, body: ScheduleUpdate) =>
      client.put<ScheduleRecord>(`/api/schedules/${seg(id)}`, { body }),
    remove: (id: string) => client.delete<undefined>(`/api/schedules/${seg(id)}`),
    share: (id: string) => client.post<ShareLink>(`/api/schedules/${seg(id)}/share`),
    unshare: (id: string) => client.delete<undefined>(`/api/schedules/${seg(id)}/share`),
    /** A schedule shared by link. Works signed out. */
    shared: (token: string) => client.get<SharedSchedule>(`/api/shared-schedules/${seg(token)}`),
  };
}

function plansApi(client: ApiClient) {
  return {
    list: () => client.get<PlanRecord[]>("/api/plans"),
    create: (body: NewPlan) => client.post<PlanRecord>("/api/plans", { body }),
    update: (id: string, body: PlanUpdate) =>
      client.put<PlanRecord>(`/api/plans/${seg(id)}`, { body }),
    remove: (id: string) => client.delete<undefined>(`/api/plans/${seg(id)}`),
  };
}

function documentsApi(client: ApiClient) {
  return {
    record: {
      get: () => client.get<AcademicRecord>("/api/record"),
      put: (body: RecordUpdate) => client.put<AcademicRecord>("/api/record", { body }),
    },
    preferences: {
      get: () => client.get<Preferences>("/api/preferences"),
      put: (body: PreferencesUpdate) => client.put<Preferences>("/api/preferences", { body }),
    },
    saved: {
      list: () => client.get<SavedCourse[]>("/api/saved"),
      add: (courseId: string) => client.put<undefined>(`/api/saved/${seg(courseId)}`),
      remove: (courseId: string) => client.delete<undefined>(`/api/saved/${seg(courseId)}`),
    },
  };
}

function friendsApi(client: ApiClient) {
  return {
    /** Accepted friends plus incoming and outgoing requests. */
    list: () => client.get<Friend[]>("/api/friends"),
    request: (andrewId: string) => client.post<Friend>("/api/friends", { body: { andrewId } }),
    /** `id` is the friendship's id. */
    accept: (id: string) => client.post<undefined>(`/api/friends/${seg(id)}/accept`),
    /** Declines, cancels or removes. */
    remove: (id: string) => client.delete<undefined>(`/api/friends/${seg(id)}`),
    invite: () => client.post<Invite>("/api/friends/invites"),
    /** Who made an invite; 404 once it expired or was used. Doesn't use it up. */
    inviteInfo: (token: string) => client.get<InviteInfo>(`/api/friends/invites/${seg(token)}`),
    redeem: (token: string) => client.post<Friend>(`/api/friends/invites/${seg(token)}/redeem`),
    revokeInvite: (token: string) => client.delete<undefined>(`/api/friends/invites/${seg(token)}`),
    sharing: () => client.get<Sharing>("/api/friends/sharing"),
    setSharing: (body: Sharing) => client.put<Sharing>("/api/friends/sharing", { body }),
    /** A friend's schedules marked visible to friends, when they share schedules. */
    schedules: (id: string, term?: string) =>
      client.get<FriendSchedule[]>(`/api/friends/${seg(id)}/schedules`, { query: { term } }),
    /** Course id to the friends taking it in `term`. */
    taking: (term: string) =>
      client.get<FriendsTaking>("/api/friends/courses", { query: { term } }),
  };
}

function finalsApi(client: ApiClient) {
  return {
    /** The exams from the student's own SIO calendar import. */
    list: (term: string, courses?: string[]) =>
      client.get<FinalExam[]>("/api/finals", { query: { term, courses: courses?.join(",") } }),
    importCalendar: (term: string, calendar: Blob) => {
      const form = new FormData();
      form.set("calendar", calendar);
      return client.post<FinalExam[]>("/api/finals/import", { form, query: { term } });
    },
    clearImport: (term: string) =>
      client.delete<undefined>("/api/finals/import", { query: { term } }),
  };
}

/** The web API's routes, bound to a client. */
export function createApi(client: ApiClient) {
  return {
    client,
    me: (signal?: AbortSignal) => client.get<Me>("/api/me", { signal }),
    /** A catalog document as JSON. Prefer lib/data's loaders, which cache and dedupe. */
    catalog: <T>(name: string, signal?: AbortSignal) =>
      client.get<T>(catalogPath(name), { signal }),
    /** Signs out on the server (the session cookie can't be cleared any other way). */
    signOut: () => client.post<undefined>("/auth/logout"),
    /** Deletes the student's data and signs out all of their sessions. */
    deleteAccount: () => client.delete<undefined>("/api/account"),
    schedules: schedulesApi(client),
    plans: plansApi(client),
    ...documentsApi(client),
    friends: friendsApi(client),
    finals: finalsApi(client),
  };
}

export type Api = ReturnType<typeof createApi>;
