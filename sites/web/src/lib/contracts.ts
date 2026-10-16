export type User = {
  subject: string;
  name: string;
  can_create: boolean;
  is_admin: boolean;
};

export type Term = {
  id: string;
  label: string;
  starts_on?: string;
  ends_on?: string;
};
export type Meeting = {
  days: string[];
  starts_at: string;
  ends_at: string;
  location?: string;
};
export type Section = {
  id: string;
  label: string;
  kind: string;
  instructors: string[];
  meetings: Meeting[];
  capacity?: number;
  enrolled?: number;
  waitlist?: number;
};
export type Fce = {
  hours_per_week?: number;
  overall_course?: number;
  overall_teaching?: number;
  semesters?: number;
};
export type Course = {
  code: string;
  name: string;
  description: string;
  units: number;
  department: string;
  level?: string;
  offered_terms: string[];
  gened_tags: string[];
  prerequisites?: string;
  corequisites?: string[];
  antirequisites?: string[];
  instructors?: string[];
  fce?: Fce;
  sections?: Section[];
  syllabi?: { label: string; url: string }[];
};
export type CourseResults = {
  items: Course[];
  total: number;
  page: number;
  page_size: number;
};
export type CourseQuery = {
  search?: string;
  department?: string;
  level?: string;
  units?: string | number;
  offered_in?: string;
  min_fce?: string | number;
  term?: string;
  view?: string;
  page?: number;
  page_size?: number;
};
export type Instructor = {
  id: string;
  name: string;
  department?: string;
  title?: string;
  email?: string;
  courses: string[];
  fce?: Fce;
};
export type InstructorResults = { items: Instructor[]; total: number };
export type Friend = {
  id: string;
  name: string;
  department?: string;
  shared_courses: string[];
  shared_schedule_id?: string;
};
export type Gened = {
  id: string;
  name: string;
  description?: string;
  courses: string[];
};
export type FinalExam = {
  course_code: string;
  course_name: string;
  section?: string;
  starts_at: string;
  ends_at: string;
  location?: string;
};
export type ScheduleItem = { course_code: string; section_ids: string[] };
export type Schedule = {
  id: string;
  name: string;
  term_id: string;
  items: ScheduleItem[];
  share_id?: string;
  updated_at?: string;
};

export class ApiError extends Error {
  constructor(public status: number, message: string) {
    super(message);
  }
}

async function json<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(path, {
    ...init,
    credentials: "same-origin",
    headers: {
      Accept: "application/json",
      ...(init?.body && !(init.body instanceof FormData)
        ? { "Content-Type": "application/json" }
        : {}),
      ...init?.headers,
    },
  });
  if (!response.ok) {
    let message = `Request failed (${response.status})`;
    try {
      message = (await response.json()).message ?? message;
    } catch { /* empty response */ }
    throw new ApiError(response.status, message);
  }
  if (response.status === 204) return undefined as T;
  return response.json() as Promise<T>;
}

const query = (params: Record<string, string | number | undefined>) => {
  const search = new URLSearchParams();
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined && value !== "") search.set(key, String(value));
  }
  return search.toString();
};

export const api = {
  me: () => json<User>("/api/me"),
  terms: () => json<Term[]>("/api/terms"),
  courses: (params: CourseQuery) =>
    json<CourseResults>(`/api/courses?${query(params)}`),
  course: (code: string, term?: string) =>
    json<Course>(`/api/courses/${encodeURIComponent(code)}?${query({ term })}`),
  instructors: (search: string) =>
    json<InstructorResults>(`/api/instructors?${query({ search })}`),
  instructor: (id: string) =>
    json<Instructor>(`/api/instructors/${encodeURIComponent(id)}`),
  geneds: () => json<Gened[]>("/api/geneds"),
  finals: (term: string, courses?: string[]) =>
    json<FinalExam[]>(
      `/api/finals?${query({ term, courses: courses?.join(",") })}`,
    ),
  uploadFinals: async (file: File) => {
    const body = new FormData();
    body.set("calendar", file);
    return json<FinalExam[]>("/api/finals/import", { method: "POST", body });
  },
  schedules: () => json<Schedule[]>("/api/schedules"),
  saveSchedule: (schedule: Schedule) =>
    json<Schedule>(`/api/schedules/${encodeURIComponent(schedule.id)}`, {
      method: "PUT",
      body: JSON.stringify(schedule),
    }),
  createSchedule: (name: string, term_id: string) =>
    json<Schedule>("/api/schedules", {
      method: "POST",
      body: JSON.stringify({ name, term_id }),
    }),
  deleteSchedule: (id: string) =>
    json<void>(`/api/schedules/${encodeURIComponent(id)}`, {
      method: "DELETE",
    }),
  sharedSchedule: (shareId: string) =>
    json<Schedule>(`/api/shared-schedules/${encodeURIComponent(shareId)}`),
  shareSchedule: (id: string) =>
    json<{ share_id: string }>(
      `/api/schedules/${encodeURIComponent(id)}/share`,
      {
        method: "POST",
      },
    ),
  friends: () => json<Friend[]>("/api/profile/friends"),
  addFriend: (andrewId: string) =>
    json<Friend>("/api/profile/friends", {
      method: "POST",
      body: JSON.stringify({ andrew_id: andrewId }),
    }),
  friendInvite: () =>
    json<{ url: string }>("/api/profile/friend-invites", { method: "POST" }),
};
