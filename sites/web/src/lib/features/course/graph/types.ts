// The requisite graph's model: what ./model.ts builds from the catalog and the student's record, and ./layout.ts
// places. Courses and groups carry the student's standing from the course pages' own checks (`evaluateRequisite()`).
import type { Unlock } from "../../../core/catalog.ts";
import type { CatalogIndex } from "../../../core/catalog-types.ts";
import type { RequisiteContext, RequisiteStatus } from "../../../core/requisites.ts";

/** How the student stands on one course. */
export type PillStatus = "met" | "in-progress" | "planned" | "missing" | "waived" | "grade-low";

/** How a group (or the line into a course) stands: the evaluation's status, or waived by an override. */
export type LinkState = RequisiteStatus | "waived";

/** Why a course is drawn: required by its parent, required alongside it, or unlocked by the course in the middle. */
export type Relation = "prereq" | "coreq" | "unlock";

export interface CourseNode {
  kind: "course";
  /** Unique within the graph: the same course can appear in several places. */
  key: string;
  id: string;
  relation: Relation;
  /** 1 for the middle course's own requisites and unlocks, 2 for theirs, and so on. */
  depth: number;
  minGrade?: string;
  /** Undefined draws a plain course: nothing to check against, or an unlock the student hasn't taken. */
  status?: PillStatus;
  /** What the status rests on, such as "Taken in Fall 2025, A". */
  note?: string;
  /** How the line into it is drawn. */
  link?: LinkState;
  /** It has requisites of its own to show (and isn't a cycle back to a course nearer the middle). */
  expandable: boolean;
  expanded: boolean;
  /** It requires a course nearer the middle that requires it: a cycle, so it isn't expanded. */
  cycle: boolean;
  /** "Expand all" showed its requisites where it first appears, so they aren't repeated here. */
  shownElsewhere?: boolean;
  /** Its own requisites, while expanded. */
  requires: GraphNode | null;
  /** Unlocks: every one of its other prerequisites is met, so the student could take it next. */
  ready?: boolean;
  /** Unlocks: every way to meet its prerequisites needs the middle course. */
  required?: boolean;
}

export type GroupOp = "all" | "any" | "atLeast" | "with" | "fork" | "requires" | "unlocks" | "dept";

export interface GroupNode {
  kind: "group";
  key: string;
  op: GroupOp;
  /** Drawn on the bracket: "AND", "OR", "2 OF", "WITH", a department; empty for a plain fork. */
  label: string;
  /** Shown after the label (departments and the unlocks). */
  count?: number;
  /** Read out and shown on hover: "One of", "Computer Science, 23 courses". */
  title: string;
  /** Undefined for groups that aren't requirements (the unlocks and their departments) or while unchecked. */
  state?: LinkState;
  /** How the line into it is drawn (the same as `state`, except for the unlocks). */
  link?: LinkState;
  relation: Relation;
  items: GraphNode[];
}

/** "+5 more" (or "Show fewer") at the end of a capped group. */
export interface MoreNode {
  kind: "more";
  key: string;
  /** The group it reveals or hides. */
  group: string;
  hidden: number;
  label: string;
  less: boolean;
  relation: Relation;
  link?: LinkState;
}

/** A condition that isn't a course ("instructor permission"). */
export interface TextNode {
  kind: "text";
  key: string;
  text: string;
  relation: Relation;
  link?: LinkState;
}

export type GraphNode = CourseNode | GroupNode | MoreNode | TextNode;

export interface GraphModel {
  rootId: string;
  /** What the middle course requires: its prerequisites, its corequisites, or both under a fork. */
  requires: GraphNode | null;
  /** What it unlocks, when shown and there are any. */
  unlocks: GroupNode | null;
}

/** The record context and how to read it. Null context: plain courses, nothing checked. */
export interface StatusInput {
  ctx: RequisiteContext | null;
  /** Nothing on record: "missing" most likely means "not entered yet", so it stays plain (as the course pages do). */
  unchecked: boolean;
  /** The semester in progress ("F26"): courses taken then without a grade yet are in progress. */
  currentTerm: string;
}

export interface Limits {
  /** Items a requirement group shows before "+N more". */
  group: number;
  /** Unlocks shown as one list up to this many; more are grouped by department. */
  flat: number;
  /** Courses each department shows before "+N more". */
  department: number;
  /** Departments shown before "+N more departments". */
  departments: number;
  /** Courses one list shows before "+N more" (a single department, or a short flat list). */
  list: number;
}

export const DEFAULT_LIMITS: Limits = {
  group: 8,
  flat: 12,
  department: 4,
  departments: 6,
  list: 40,
};

export interface BuildOptions extends StatusInput {
  index: CatalogIndex;
  rootId: string;
  /** Course nodes the student expanded, by key. */
  expanded: ReadonlySet<string>;
  /** Course nodes the student collapsed after "Expand all", by key. */
  collapsed?: ReadonlySet<string>;
  /**
   * "Expand all": courses down to this depth are expanded, each course once (where it first appears, depth first);
   * its other appearances say it's shown elsewhere. 0 expands only what the student asked for.
   */
  autoDepth?: number;
  /** Groups whose items past the cap are shown. */
  revealed: ReadonlySet<string>;
  /** Already filtered; null hides the unlocks. */
  unlocks: readonly Unlock[] | null;
  /** Names a course-number prefix ("15") as its department. */
  departmentOf: (prefix: string) => string;
  limits?: Partial<Limits>;
}

/** A walk over the model while it's built. */
export interface Walk {
  options: BuildOptions;
  limits: Limits;
  /** Courses expanded so far, so "Expand all" shows each one's requisites once. */
  opened: Set<string>;
}
