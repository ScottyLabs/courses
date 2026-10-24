// Final exams: the registrar's schedule (`finals/<term>`, docs/src/courseweave.md) matched to a student's schedule,
// the student's own SIO calendar import, warnings about exams that collide or crowd together, and a calendar file.
// Times are Pittsburgh wall-clock time: a date (YYYY-MM-DD) and minutes after midnight.
import type { FinalExam } from "../../api/types.ts";
import type { RegistrarFinal } from "../../data/documents.ts";
import { easternOffsetHours, escapeText, foldLine } from "../../core/ics.ts";
import { codeOf } from "../../core/requisite-format.ts";
import type { ResolvedEntry } from "../../core/state/schedule-view.ts";

/** One exam, from either source, in Pittsburgh time. */
export interface Exam {
  courseId: string;
  code: string;
  /** The section the registrar lists it for; null for an SIO import. */
  section: string | null;
  title: string;
  date: string;
  start: number;
  end: number;
  location: string | null;
  mode: string | null;
  source: "registrar" | "sio";
}

/** A course in the student's schedule and its exam. */
export interface MyFinal {
  courseId: string;
  code: string;
  title: string;
  color: number;
  /** The exam, when one was found. */
  exam: Exam | null;
  /**
   * section: the registrar lists the student's section; course: every section has the same exam; sio: from the
   * student's SIO calendar; pick: choose a section first; none: no exam listed for this course or section.
   */
  match: "section" | "course" | "sio" | "pick" | "none";
}

export interface FinalWarning {
  kind: "overlap" | "back-to-back" | "three-in-24";
  /** Course codes involved, in exam order. */
  codes: string[];
  message: string;
}

const DAY_MINUTES = 1440;

function dayNumber(date: string): number {
  const [y = 1970, m = 1, d = 1] = date.split("-").map(Number);
  return Date.UTC(y, m - 1, d) / 86_400_000;
}

/** Minutes since the epoch in Pittsburgh wall-clock time, for ordering and gaps. */
export function examMinutes(exam: Pick<Exam, "date" | "start">): number {
  return dayNumber(exam.date) * DAY_MINUTES + exam.start;
}

export function fromRegistrar(f: RegistrarFinal): Exam {
  return {
    courseId: f.courseId,
    code: codeOf(f.courseId),
    section: f.section,
    title: f.title,
    date: f.date,
    start: f.start,
    end: f.end,
    location: f.location === "" ? null : f.location,
    mode: f.mode,
    source: "registrar",
  };
}

/** Pittsburgh date and minutes for an RFC 3339 instant. */
function local(instant: string): { date: string; minutes: number } {
  const t = Date.parse(instant);
  const utcDay = Math.floor(t / 86_400_000) * 86_400_000;
  const shifted = new Date(t - easternOffsetHours(utcDay) * 3_600_000);
  const date = shifted.toISOString().slice(0, 10);
  return { date, minutes: shifted.getUTCHours() * 60 + shifted.getUTCMinutes() };
}

export function fromSio(f: FinalExam): Exam {
  const start = local(f.startsAt);
  const end = local(f.endsAt);
  return {
    courseId: f.courseId,
    code: f.courseCode,
    section: null,
    title: f.title,
    date: start.date,
    start: start.minutes,
    end: end.date === start.date ? end.minutes : DAY_MINUTES,
    location: f.location === "" ? null : f.location,
    mode: null,
    source: "sio",
  };
}

/** The registrar's exams by course id. */
export function byCourse(finals: readonly RegistrarFinal[]): Map<string, RegistrarFinal[]> {
  const out = new Map<string, RegistrarFinal[]>();
  for (const f of finals) out.set(f.courseId, [...(out.get(f.courseId) ?? []), f]);
  return out;
}

function sameTime(a: RegistrarFinal, b: RegistrarFinal): boolean {
  return a.date === b.date && a.start === b.start && a.end === b.end;
}

/**
 * The exam for each visible course in a schedule. The registrar lists exams by lecture (or standalone section): the
 * chosen lecture's exam, else the chosen recitation's, else the course's when every section shares one time. The
 * student's SIO import fills in courses the registrar's file doesn't have.
 */
export function matchFinals(
  entries: readonly ResolvedEntry[],
  registrar: readonly RegistrarFinal[] | null,
  sio: readonly FinalExam[] = [],
): MyFinal[] {
  const index = byCourse(registrar ?? []);
  return entries
    .filter((e) => !e.entry.hidden)
    .map((e) => matchOne(e, index.get(e.course.id) ?? [], sio));
}

function matchOne(
  e: ResolvedEntry,
  listed: readonly RegistrarFinal[],
  sio: readonly FinalExam[],
): MyFinal {
  const base = {
    courseId: e.course.id,
    code: e.course.code,
    title: e.course.title,
    color: e.entry.color,
  };
  const found = (exam: Exam | null, match: MyFinal["match"]): MyFinal =>
    Object.assign(base, { exam, match });
  const exact =
    listed.find((f) => f.section === e.section?.name) ??
    listed.find((f) => e.child !== null && f.section === e.child.name);
  if (exact !== undefined) return found(fromRegistrar(exact), "section");
  const first = listed[0];
  if (first !== undefined && listed.every((f) => sameTime(f, first)))
    return found(fromRegistrar(first), "course");
  const imported = sio.find((f) => f.courseId === e.course.id);
  if (imported !== undefined) return found(fromSio(imported), "sio");
  return found(null, listed.length > 0 && e.section === null ? "pick" : "none");
}

function timed(list: readonly MyFinal[]): (MyFinal & { exam: Exam })[] {
  return list
    .filter((f): f is MyFinal & { exam: Exam } => f.exam !== null && f.exam.mode !== "Cancelled")
    .toSorted((a, b) => examMinutes(a.exam) - examMinutes(b.exam));
}

/**
 * Exams that overlap, exams back to back (the next starts within three hours of the previous one's end, on the
 * same day), and three or more exams starting within 24 hours, which CMU lets students ask to reschedule.
 */
export function finalWarnings(list: readonly MyFinal[], backToBackMinutes = 180): FinalWarning[] {
  const exams = timed(list);
  const out: FinalWarning[] = [];
  for (let i = 0; i < exams.length; i++) {
    for (let j = i + 1; j < exams.length; j++) {
      const a = exams[i];
      const b = exams[j];
      if (a === undefined || b === undefined) continue;
      const aStart = examMinutes(a.exam);
      const aEnd = aStart + (a.exam.end - a.exam.start);
      const bStart = examMinutes(b.exam);
      if (bStart < aEnd)
        out.push({
          kind: "overlap",
          codes: [a.code, b.code],
          message: `${a.code} and ${b.code} exams overlap.`,
        });
      else if (a.exam.date === b.exam.date && bStart - aEnd <= backToBackMinutes)
        out.push({
          kind: "back-to-back",
          codes: [a.code, b.code],
          message: `${a.code} and ${b.code} exams are back to back.`,
        });
    }
  }
  // Each run of three or more starting within 24 hours of an exam, reported once (the longest from its start).
  let coveredUntil = -1;
  for (let i = 0; i < exams.length; i++) {
    const first = exams[i];
    if (first === undefined) continue;
    const from = examMinutes(first.exam);
    const run = exams.slice(i).filter((f) => examMinutes(f.exam) < from + DAY_MINUTES);
    const last = i + run.length - 1;
    if (run.length >= 3 && last > coveredUntil) {
      coveredUntil = last;
      const codes = run.map((f) => f.code);
      out.push({
        kind: "three-in-24",
        codes,
        message: `${run.length} exams within 24 hours: ${codes.join(", ")}. You can ask to reschedule one.`,
      });
    }
  }
  return out;
}

function pad(n: number): string {
  return String(n).padStart(2, "0");
}

function localStamp(date: string, minutes: number): string {
  return `${date.replaceAll("-", "")}T${pad(Math.floor(minutes / 60))}${pad(minutes % 60)}00`;
}

/** A .ics file with one event per exam, in Pittsburgh time. */
export function finalsIcs(exams: readonly Exam[], calendarName: string, now = new Date()): string {
  const stamp = `${now.toISOString().replaceAll(/[-:]/gu, "").slice(0, 15)}Z`;
  const lines = [
    "BEGIN:VCALENDAR",
    "VERSION:2.0",
    "PRODID:-//ScottyLabs//CMU Courses//EN",
    "CALSCALE:GREGORIAN",
    "METHOD:PUBLISH",
    `X-WR-CALNAME:${escapeText(calendarName)}`,
    "X-WR-TIMEZONE:America/New_York",
    "BEGIN:VTIMEZONE",
    "TZID:America/New_York",
    "BEGIN:DAYLIGHT",
    "TZOFFSETFROM:-0500",
    "TZOFFSETTO:-0400",
    "TZNAME:EDT",
    "DTSTART:19700308T020000",
    "RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=2SU",
    "END:DAYLIGHT",
    "BEGIN:STANDARD",
    "TZOFFSETFROM:-0400",
    "TZOFFSETTO:-0500",
    "TZNAME:EST",
    "DTSTART:19701101T020000",
    "RRULE:FREQ=YEARLY;BYMONTH=11;BYDAY=1SU",
    "END:STANDARD",
    "END:VTIMEZONE",
  ];
  for (const exam of exams) {
    if (exam.mode === "Cancelled") continue;
    lines.push(
      "BEGIN:VEVENT",
      `UID:final-${exam.courseId}-${exam.section ?? "sio"}-${exam.date}@cmucourses`,
      `DTSTAMP:${stamp}`,
      `DTSTART;TZID=America/New_York:${localStamp(exam.date, exam.start)}`,
      `DTEND;TZID=America/New_York:${localStamp(exam.date, exam.end)}`,
      `SUMMARY:${escapeText(`${exam.code} final exam`)}`,
    );
    if (exam.location !== null) lines.push(`LOCATION:${escapeText(exam.location)}`);
    lines.push(`DESCRIPTION:${escapeText(exam.title)}`, "END:VEVENT");
  }
  lines.push("END:VCALENDAR");
  return `${lines.map((l) => foldLine(l)).join("\r\n")}\r\n`;
}

/** "Monday, December 7". */
export function examDay(date: string): string {
  const [y = 1970, m = 1, d = 1] = date.split("-").map(Number);
  return new Date(Date.UTC(y, m - 1, d)).toLocaleDateString("en-US", {
    weekday: "long",
    month: "long",
    day: "numeric",
    timeZone: "UTC",
  });
}

/** Whether the exam's location is a room rather than the registrar's note ("To Be Announced"). */
export function hasRoom(exam: Pick<Exam, "location">): boolean {
  return exam.location !== null && !/^to be|^tba$/iu.test(exam.location);
}
