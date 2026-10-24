// The academic record (`/api/record`): courses taken, transfer and exam credit, placements and requisite overrides.
// Courseweave's record store, as one server record per student. SIO and Stellic PDFs are parsed in the browser
// (core/record/SioImportPdf.ts); only the parsed courses reach the server.
import type { Api } from "../api/endpoints.ts";
import type { AcademicRecord } from "../api/types.ts";
import type { RequisiteOverride } from "../core/requisites.ts";
import {
  addEntries,
  EMPTY_RECORD,
  recordEdits,
  recordIsEmpty,
  recordTotals,
  type NewRecordEntry,
  type RecordData,
  type RecordEntry,
  type RecordTotals,
  type StoredOverride,
  type StudentProfile,
} from "../core/state/record.ts";
import { mergeRecordDoc, normalizeRecordDoc } from "../core/sync-merge.ts";
import type { StoreDeps } from "./drafts.ts";
import { DocumentStore } from "./document.ts";
import type { Stored } from "./sync.ts";

function fromApi(r: AcademicRecord): Stored<RecordData> {
  return { value: normalizeRecordDoc(r) ?? EMPTY_RECORD, revision: r.revision };
}

export class RecordStore {
  readonly store: DocumentStore<RecordData>;
  readonly #now: () => number;

  constructor(api: Api, deps: StoreDeps) {
    this.#now = () => deps.env.now();
    this.store = new DocumentStore<RecordData>(
      {
        name: "record",
        empty: () => EMPTY_RECORD,
        get: async () => fromApi(await api.record.get()),
        put: async (value, revision) => fromApi(await api.record.put({ ...value, revision })),
        normalize: normalizeRecordDoc,
        merge: mergeRecordDoc,
        isEmpty: (r) => recordIsEmpty(r) && Object.keys(r.profile).length === 0,
      },
      deps,
    );
  }

  get data(): RecordData {
    return this.store.value;
  }

  get entries(): RecordEntry[] {
    return this.store.value.entries;
  }

  /** Course id to waived requisites for that course (instructor permission, advisor override). */
  get overrides(): Record<string, StoredOverride> {
    return this.store.value.overrides;
  }

  get profile(): StudentProfile {
    return this.store.value.profile;
  }

  /** Nothing recorded yet (the onboarding prompt asks for a record). */
  get isEmpty(): boolean {
    return recordIsEmpty(this.store.value);
  }

  /** Units by source, earned units and QPA, given catalog units per course. */
  totals(unitsOf: (courseId: string) => number | null | undefined): RecordTotals {
    return recordTotals(this.store.value.entries, unitsOf);
  }

  addEntry(entry: NewRecordEntry): string {
    return this.addEntries([entry])[0] ?? "";
  }

  /** Adds entries; returns their new ids, in order. */
  addEntries(entries: NewRecordEntry[]): string[] {
    // Stamped once, outside the edit, so a replay after a conflict adds the same entries with the same ids.
    const { data, ids } = addEntries(EMPTY_RECORD, entries, this.#now());
    if (ids.length > 0)
      this.store.edit((current) => ({
        ...current,
        entries: [...current.entries, ...data.entries],
      }));
    return ids;
  }

  updateEntry(id: string, patch: Partial<NewRecordEntry>): void {
    this.store.edit(recordEdits.updateEntry(id, patch));
  }

  removeEntry(id: string): void {
    this.store.edit(recordEdits.removeEntry(id));
  }

  /** Removes every entry produced by one exam. */
  removeExam(examId: string): void {
    this.store.edit(recordEdits.removeExam(examId));
  }

  /** Merges flags into a course's override; `null` clears it. Clearing every flag removes it. */
  setOverride(courseId: string, patch: Partial<RequisiteOverride> | null): void {
    this.store.edit(recordEdits.setOverride(courseId, patch, this.#now()));
  }

  setProfile(patch: Partial<StudentProfile>): void {
    this.store.edit(recordEdits.setProfile(patch));
  }

  /** Replaces the record, or merges an imported one in. */
  importRecord(data: RecordData, mode: "replace" | "merge"): void {
    this.store.edit(recordEdits.importRecord(data, mode));
  }

  clearRecord(): void {
    this.store.edit(recordEdits.clearRecord());
  }
}
