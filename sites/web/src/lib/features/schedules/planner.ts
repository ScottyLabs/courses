// The schedule page's calendar and lists, from Courseweave's PlannerCalendar, UnscheduledList and ExportMenu: which
// blocks go on the week calendar (placed sections, alternatives for a clicked course, a previewed course, a
// friend's schedule), what has no set time, and the plain-text course list. Pure functions over a ScheduleView.
import type { TbaItem } from "../../components/calendar/types.ts";
import { isUnscheduled } from "../../core/blocks.ts";
import { nextColor } from "../../core/colors.ts";
import type { Term } from "../../core/data.ts";
import {
  conflictingCodes,
  joinList,
  meetingSummary,
  partBlocks,
  partLabel,
  unitsText,
} from "../../core/planner/format.ts";
import { coursePath } from "../../core/routes.ts";
import { childSections, topSections } from "../../core/sections.ts";
import type { ResolvedEntry, ScheduleView } from "../../core/state/schedule-view.ts";
import { resolveEntries } from "../../core/state/schedule-view.ts";
import type { Schedule } from "../../core/state/store.ts";
import { formatUnits } from "../../core/time.ts";
import type { Section } from "../../core/types.ts";
import type { CalendarBlock } from "../../core/week-grid.ts";

/** A course being considered but not yet chosen: specific section names, or every lecture/section when null. */
export interface Preview {
  courseId: string;
  sections: string[] | null;
}

/** A placed course clicked on the calendar; its alternatives at that level show as outlines. */
export interface CalendarSelection {
  courseId: string;
  /** section: other lectures or sections. child: other recitations of the current lecture. */
  level: "section" | "child";
}

export interface PlannerCallbacks {
  select(selection: CalendarSelection | null): void;
  switchSection(courseId: string, section: string): void;
  switchChild(courseId: string, child: string): void;
}

/** Whether a section runs in the shown session: everything when `mini` is null, else full-semester and that mini. */
export function inSession(section: Section, mini: number | null): boolean {
  return mini === null || section.mini === null || section.mini === mini;
}

interface BlockOptions {
  selected: CalendarSelection | null;
  preview: Preview | null;
  mini: number | null;
  on: PlannerCallbacks;
}

/** The schedule's own sections; clicking one selects it (or clears the selection). */
function placedBlocks(view: ScheduleView, { selected, mini, on }: BlockOptions): CalendarBlock[] {
  const blocks: CalendarBlock[] = [];
  for (const { entry, course, section, child, parts } of view.placed) {
    for (const part of parts) {
      if (!inSession(part, mini)) continue;
      const level = part === section ? "section" : "child";
      const isSelected = selected?.courseId === course.id && selected.level === level;
      const alternatives =
        level === "section"
          ? topSections(course).length > 1
          : child !== null && section !== null && childSections(course, section.name).length > 1;
      const kind = part.kind === "lecture" ? "lectures" : "sections";
      const others = level === "section" ? kind : "recitations";
      blocks.push(
        ...partBlocks(course, part, entry.color, {
          conflict: view.conflicts.has(course.id),
          selected: isSelected,
          onClick: () => {
            on.select(isSelected ? null : { courseId: course.id, level });
          },
          tooltip: `${course.code} ${course.title}, ${partLabel(part)}.${alternatives ? ` Select to see other ${others}.` : ""}`,
        }),
      );
    }
  }
  return blocks;
}

/** The selected course's other sections at the selected level, outlined; clicking one switches to it. */
function alternativeBlocks(
  view: ScheduleView,
  { selected, mini, on }: BlockOptions,
): CalendarBlock[] {
  if (selected === null) return [];
  const chosen = view.entries.find((e) => e.course.id === selected.courseId && !e.entry.hidden);
  if (chosen === undefined) return [];
  const { course, entry, section, child } = chosen;
  let choices: Section[] = [];
  if (selected.level === "section")
    choices = topSections(course).filter((s) => s.name !== section?.name);
  else if (section !== null)
    choices = childSections(course, section.name).filter((s) => s.name !== child?.name);
  return choices
    .filter((s) => inSession(s, mini))
    .flatMap((s) => {
      const conflicts = conflictingCodes(s, course.id, view);
      const clash = conflicts.length > 0 ? `, conflicts with ${joinList(conflicts)}` : "";
      return partBlocks(course, s, entry.color, {
        keyPrefix: "ghost-",
        variant: "ghost",
        ...(conflicts.length > 0 ? { detail: `${partLabel(s)}${clash}` } : {}),
        tooltip: `Switch ${course.code} to ${partLabel(s)} (${meetingSummary(s)})${clash}`,
        onClick: () => {
          if (selected.level === "section") on.switchSection(course.id, s.name);
          else on.switchChild(course.id, s.name);
        },
      });
    });
}

/** A course from the search, or a hovered section option, hatched. */
function previewBlocks(
  view: ScheduleView,
  term: Term,
  { preview, mini }: BlockOptions,
): CalendarBlock[] {
  if (preview === null) return [];
  const course = term.courseById.get(preview.courseId);
  if (course === undefined) return [];
  const entry = view.entries.find((e) => e.course.id === course.id)?.entry;
  const color = entry?.color ?? nextColor(view.entries.map((e) => e.entry.color));
  const sections =
    preview.sections === null
      ? topSections(course)
      : course.sections.filter((s) => preview.sections?.includes(s.name) === true);
  return sections
    .filter((s) => inSession(s, mini))
    .flatMap((s) => partBlocks(course, s, color, { keyPrefix: "preview-", variant: "preview" }));
}

/** Blocks for the schedule: placed sections, the selected course's alternatives, and the previewed course. */
export function plannerBlocks(
  view: ScheduleView,
  term: Term,
  options: BlockOptions,
): CalendarBlock[] {
  return [
    ...placedBlocks(view, options),
    ...alternativeBlocks(view, options),
    ...previewBlocks(view, term, options),
  ];
}

/** Blocks for someone else's schedule shown under one's own (a friend's), dimmed and labeled with their name. */
export function overlayBlocks(
  schedule: Pick<Schedule, "entries">,
  term: Term,
  owner: string,
  mini: number | null,
): CalendarBlock[] {
  const fake: Schedule = {
    id: "overlay",
    termId: term.meta.id,
    name: owner,
    entries: schedule.entries,
    createdAt: 0,
    updatedAt: 0,
  };
  return resolveEntries(fake, term)
    .filter((e) => !e.entry.hidden)
    .flatMap((e) =>
      e.parts
        .filter((p) => inSession(p, mini))
        .flatMap((p) =>
          partBlocks(e.course, p, 9, {
            keyPrefix: "friend-",
            variant: "preview",
            label: e.course.code,
            section: owner,
            tooltip: `${owner}: ${e.course.code} ${e.course.title}, ${partLabel(p)}`,
          }),
        ),
    );
}

/** Visible courses with a chosen section (or recitation) that has no set meeting time. */
export function tbaItems(view: ScheduleView, termId: string): TbaItem[] {
  return view.placed
    .map((e) => ({ e, unscheduled: e.parts.filter((p) => isUnscheduled(p)) }))
    .filter(({ unscheduled }) => unscheduled.length > 0)
    .map(({ e, unscheduled }) => ({
      key: e.course.id,
      label: e.course.code,
      title: e.course.title,
      detail: `${unscheduled.map((p) => partLabel(p)).join(", ")} \u00B7 ${unitsText(e.course.units)}`,
      color: e.entry.color,
      href: coursePath(e.course.code, termId),
    }));
}

/** How many pairs of courses conflict. */
export function conflictCount(view: ScheduleView): number {
  return [...view.conflicts.values()].reduce((n, list) => n + list.length, 0) / 2;
}

/** The codes of the courses `entry` conflicts with. */
export function conflictCodes(entry: ResolvedEntry, view: ScheduleView): string[] {
  return (view.conflicts.get(entry.course.id) ?? []).map(
    (id) => view.entries.find((e) => e.course.id === id)?.course.code ?? id,
  );
}

/** The schedule as plain text, one course a line, with sections and times. */
export function courseListText(view: ScheduleView, name: string, termName: string): string {
  const lines = [`${name} (${termName})`];
  for (const { course, section, parts, factorable, needsChild, entry } of view.entries) {
    if (entry.hidden) continue;
    const units = `${course.units === null ? "variable units" : `${formatUnits(course.units)} units`}${factorable ? "" : " (pass/fail)"}`;
    const times = parts.map((p) => `${partLabel(p)}: ${meetingSummary(p)}`).join("; ");
    lines.push(
      section === null
        ? `${course.code} ${course.title}, ${units}, no section chosen`
        : `${course.code} ${section.title ?? course.title}, ${units}, ${times}${needsChild ? "; recitation not chosen" : ""}`,
    );
  }
  const { total, factorable, nonFactorable, hasVariable } = view.units;
  lines.push(
    `Total: ${total} units${hasVariable ? " plus variable-unit courses" : ""}${nonFactorable > 0 ? ` (${factorable} factorable, ${nonFactorable} pass/fail)` : ""}`,
  );
  return lines.join("\n");
}

/** A file name from a schedule name: no characters file systems refuse. */
export function fileName(name: string): string {
  return (
    name
      .replaceAll(/[\\/:*?"<>|]+/gu, " ")
      .replaceAll(/\s+/gu, " ")
      .trim() || "schedule"
  );
}
