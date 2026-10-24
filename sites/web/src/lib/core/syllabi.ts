/* oxlint-disable unicorn/no-array-callback-reference -- Courseweave's code, ported unchanged (decision 14) so fixes carry over between the two codebases */
// Past syllabi for a course: the index (the `syllabi/<NN>` catalog document, loaded by lib/data) and links to the
// files (/api/syllabi/<key>). Both need CMU sign-in, like FCEs; see syllabus-files.ts.
export type { SyllabiChunk, SyllabusEntry } from "./syllabus-files.ts";

/** Where a hosted syllabus opens (PDFs show in the browser) or downloads. */
export function syllabusHref(key: string, download = false, base = "/"): string {
  const root = base.endsWith("/") ? base : `${base}/`;
  return `${root}api/syllabi/${key.split("/").map(encodeURIComponent).join("/")}${download ? "?download=1" : ""}`;
}
