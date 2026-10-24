// What every part of the degree plan page reads: the plan, everything derived from it, lookups and the page's
// actions. PlanView provides it; the semester grid, term cards, rows and the requirements panel read it.
import { getContext, setContext } from "svelte";
import type { CatalogIndex } from "$lib/core/catalog-types";
import type { Term } from "$lib/core/data";
import type { PlanDerived } from "$lib/core/degree/hooks";
import type { Plan, RecordLike } from "$lib/core/plan";
import type { AuditResult } from "$lib/core/plan-types";
import type { CourseInfo } from "$lib/core/record/hooks";

export type PlanLayout = "cards" | "table";

/** A course being dragged: a planned course moving, or a course from the requirements being added. */
export type DragItem = { kind: "move"; planCourseId: string } | { kind: "add"; courseId: string };

export interface PlanPageContext {
  readonly plan: Plan;
  readonly derived: PlanDerived;
  readonly catalog: CatalogIndex | null;
  /** The semester being browsed (the `term` parameter), for titles and units the catalog lacks. */
  readonly routed: Term | null;
  readonly record: RecordLike;
  readonly audit: AuditResult | null;
  /** Term to the course ids its published schedule lists, for the semesters that have one. */
  readonly scheduled: Map<string, Set<string>>;
  /** Terms with a published schedule (and so a schedule planner). */
  readonly published: Set<string>;
  readonly layout: PlanLayout;
  /** Drag and drop is on (a mouse or trackpad on a wide screen); menus do the same everywhere. */
  readonly dragEnabled: boolean;
  /** Phones: semesters collapse. */
  readonly compact: boolean;
  /** The course being dragged, if any. */
  readonly dragging: DragItem | null;
  lookup(courseId: string): CourseInfo;
  /** A course's page, in the semester it's planned for when that semester offers it. */
  href(courseId: string, term?: string | null): string;
  fceHours(courseId: string): number | null;
  /** Whether a semester's card is expanded (phones collapse past semesters). */
  isOpen(term: string): boolean;
  toggleTerm(term: string): void;
  /** Opens a course's semester and scrolls to it, focused. */
  jumpTo(planCourseId: string, term: string): void;
  /** Adds a course to the semester the plan suggests, then jumps to it. */
  addSuggested(courseId: string): void;
  startDrag(item: DragItem): void;
  endDrag(): void;
}

const KEY = Symbol("plan-page");

export function setPlanPage(context: PlanPageContext): void {
  setContext(KEY, context);
}

export function planPage(): PlanPageContext {
  return getContext<PlanPageContext>(KEY);
}
