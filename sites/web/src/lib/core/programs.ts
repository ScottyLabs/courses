/* oxlint-disable typescript/strict-boolean-expressions, unicorn/explicit-length-check -- Courseweave's code, ported unchanged (decision 14) so fixes carry over between the two codebases */
// Choosing degree program versions and running audits. lib/data loads the `programs/*` documents.

import { relatedAuditIds, runAudit } from "./audit.ts";
import type { Loadable } from "./catalog.ts";
import type { CatalogIndex } from "./catalog-types.ts";
import type { ProgramVersion } from "./degree-types.ts";
import type { AuditInput, AuditResult, ProgramInfo, ProgramVersionInfo } from "./plan-types.ts";

/** Selected versions plus everything they bring in (college gen-ed, chosen concentrations). */
export async function loadAuditVersions(
  input: AuditInput,
  loadProgramVersion: (auditId: number) => Promise<ProgramVersion>,
): Promise<Map<number, ProgramVersion>> {
  const versions = new Map<number, ProgramVersion>();
  const selected = await Promise.all(input.programs.map((p) => loadProgramVersion(p.auditId)));
  for (const v of selected) versions.set(v.auditId, v);
  const related = await Promise.all(
    relatedAuditIds(input, versions).map((id) => loadProgramVersion(id).catch(() => null)),
  );
  for (const v of related) if (v) versions.set(v.auditId, v);
  return versions;
}

/** The best version of a program for a student who entered in `entryYear` (newest Pittsburgh otherwise). */
export function defaultVersion(
  program: ProgramInfo,
  entryYear?: number | null,
  campus = "Pittsburgh",
): ProgramVersionInfo | null {
  const versions = program.versions;
  if (!versions.length) return null;
  const local = versions.filter((v) => !v.campus || v.campus === campus);
  const pool = local.length ? local : versions;
  if (entryYear) {
    const exact = pool.find((v) => v.entryYear === entryYear);
    if (exact) return exact;
    const before = pool.find((v) => v.entryYear !== null && v.entryYear <= entryYear);
    if (before) return before;
  }
  return pool[0];
}

// ---------------------------------------------------------------------------

export interface AuditData {
  result: AuditResult;
  /** Every version audited, including auto-attached ones. */
  versions: Map<number, ProgramVersion>;
  catalog: CatalogIndex | null;
}

/** Audits `input` against loaded versions; an error result instead of a throw. */
export function auditData(
  input: AuditInput,
  versions: Map<number, ProgramVersion>,
  catalog: CatalogIndex | null,
): Loadable<AuditData> {
  try {
    return {
      status: "ready",
      value: { result: runAudit(input, versions, catalog), versions, catalog },
    };
  } catch (e) {
    return { status: "error", error: e instanceof Error ? e.message : String(e) };
  }
}
