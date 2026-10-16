import type {
  Course,
  CourseQuery,
  CourseResults,
  Gened,
  Instructor,
  InstructorResults,
  Term,
  User,
} from "./contracts";

/** Undefined means this dataset/query is not covered; use the remote API. */
export interface LocalCatalog {
  terms(): Promise<Term[] | undefined>;
  courses(query: CourseQuery): Promise<CourseResults | undefined>;
  course(code: string, term?: string): Promise<Course | undefined>;
  instructors(search: string): Promise<InstructorResults | undefined>;
  instructor(id: string): Promise<Instructor | undefined>;
  geneds(): Promise<Gened[] | undefined>;
  dispose(): void;
}

export type LocalCatalogFactory = (
  user: User,
  signal: AbortSignal,
) => Promise<LocalCatalog>;
export type CatalogState = {
  status: "idle" | "loading" | "ready" | "unavailable";
  error?: string;
};
