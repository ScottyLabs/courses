// The catalog documents `GET /api/catalog/<name>` serves (decision 16; docs/src/courseweave.md, "Catalog documents"),
// in Courseweave's formats. "Member" documents need a signed-in CMU user; public versions of `term/<id>` and
// `finals/<id>` leave out instructors and rooms.
import type { CatalogDetails, CatalogIndex } from "../core/catalog-types.ts";
import type { ProgramVersion } from "../core/degree-types.ts";
import type { FceCoursesFile, FceInstructorsFile, FceSummaryFile } from "../core/fce.ts";
import type { ProgramsIndex } from "../core/plan-types.ts";
import type { SyllabiChunk } from "../core/syllabus-files.ts";
import type { OfferingsData, TeachingData, TermData, TermIndexEntry } from "../core/types.ts";

/** The `build` document: what the catalog was built from. */
export interface BuildInfo {
  [key: string]: unknown;
}

/** One exam in the registrar's final exam schedule (`finals/<termId>`). */
export interface RegistrarFinal {
  courseId: string;
  section: string;
  title: string;
  /** YYYY-MM-DD. */
  date: string;
  /** Minutes after midnight in Pittsburgh. */
  start: number;
  end: number;
  mode: string;
  location: string;
  /** Empty in the public version. */
  instructors: string[];
}

/** Course-number prefix of a course id: "15" for "15122". Large documents are split by it. */
export function prefixOf(courseId: string): string {
  return courseId.slice(0, 2);
}

/** Document names, as `GET /api/catalog/<name>` takes them. */
export const docs = {
  build: "build",
  terms: "terms",
  term: (id: string) => `term/${id}`,
  offerings: "offerings",
  catalogIndex: "catalog/index",
  catalogDetails: (prefix: string) => `catalog/details/${prefix}`,
  teaching: "teaching",
  fceSummary: "fce/summary",
  fceCoursesIndex: "fce/courses/index",
  fceCourses: (prefix: string) => `fce/courses/${prefix}`,
  fceInstructors: "fce/instructors",
  programsIndex: "programs/index",
  program: (auditId: number) => `programs/${auditId}`,
  syllabi: (prefix: string) => `syllabi/${prefix}`,
  finals: (termId: string) => `finals/${termId}`,
} as const;

/** Documents that only signed-in members can load (signed out they answer 401). */
export function isMemberOnly(name: string): boolean {
  return /^(?:teaching|fce\/.*|programs\/.*|syllabi\/.*)$/u.test(name);
}

/** What each document parses to. */
export interface DocumentTypes {
  build: BuildInfo;
  terms: TermIndexEntry[];
  term: TermData;
  offerings: OfferingsData;
  catalogIndex: CatalogIndex;
  catalogDetails: Record<string, CatalogDetails>;
  teaching: TeachingData;
  fceSummary: FceSummaryFile;
  fceCoursesIndex: string[];
  fceCourses: FceCoursesFile;
  fceInstructors: FceInstructorsFile;
  programsIndex: ProgramsIndex;
  program: ProgramVersion;
  syllabi: SyllabiChunk;
  finals: RegistrarFinal[];
}
