// Grade choices for courses taken, and what a grade means for QPA (Courseweave's features/record/TakenSection.tsx).
import type { CourseInfo } from "$lib/core/record/hooks";
import { isFactorableGrade } from "$lib/core/requisites";
import type { Option } from "$lib/components/ui/types";

export const GRADE_OPTIONS: Option[] = [
  { value: "", label: "No grade yet" },
  ...["A", "B", "C", "D"].map((g) => ({ value: g, label: g })),
  { value: "R", label: "R (failed)" },
  { value: "P", label: "P (pass)" },
  { value: "N", label: "N (no pass)" },
  { value: "S", label: "S (satisfactory)" },
  { value: "W", label: "W (withdrew)" },
  { value: "I", label: "I (incomplete)" },
  { value: "O", label: "O (audit)" },
  ...["A+", "A-", "B+", "B-", "C+", "C-", "D+"].map((g) => ({
    value: g,
    label: `${g} (graduate scale)`,
  })),
];

/** A note under a course whose grade doesn't count toward QPA, or whose course is graded pass/fail only. */
export function gradeNote(grade: string | undefined, info: CourseInfo): string | null {
  const factor = isFactorableGrade(grade);
  if (info.passFail && factor === true) return "This course is graded pass/fail only.";
  if (factor === false) return "Not factored into QPA";
  return null;
}

/** Grade options that keep an imported grade the list doesn't have (such as "AD") selectable. */
export function gradeOptionsWith(grade: string | undefined): Option[] {
  if (grade === undefined || grade === "" || GRADE_OPTIONS.some((o) => o.value === grade))
    return GRADE_OPTIONS;
  return [...GRADE_OPTIONS, { value: grade, label: grade }];
}
