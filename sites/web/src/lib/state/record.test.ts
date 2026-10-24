/* oxlint-disable typescript/no-floating-promises, typescript/no-non-null-assertion, typescript/no-unnecessary-type-assertion, eslint/max-lines-per-function -- Courseweave's state/record.test.ts, ported to these stores (decision 14); node:test awaits describe and it itself */
import { beforeEach, describe, it } from "node:test";
import type { CatalogIndex } from "../core/catalog-types.ts";
import { buildRequisiteContext } from "../core/record/context.ts";
import { checkCourse } from "../core/requisites.ts";
import { exportRecord, parseRecordExport, recordTotals } from "../core/state/record.ts";
import { expect } from "../testing.ts";
import type { RecordStore } from "./record.ts";
import { draftStores } from "./test-env.ts";

const units: Record<string, number> = { "15122": 12, "21122": 10, "21120": 10, "15213": 12 };
const unitsOf = (id: string) => units[id];

const catalog: CatalogIndex = {
  generatedAt: "",
  courses: {
    "15213": {
      id: "15213",
      code: "15-213",
      title: "Introduction to Computer Systems",
      units: 12,
      prereqs: { type: "course", id: "15122", minGrade: "C" },
      coreqs: [],
      antireqs: [],
      equivalents: [],
      passFail: false,
    },
    "21259": {
      id: "21259",
      code: "21-259",
      title: "Calculus in Three Dimensions",
      units: 10,
      prereqs: { type: "course", id: "21122" },
      coreqs: [],
      antireqs: [],
      equivalents: [],
      passFail: false,
    },
  },
};

let record: RecordStore;
beforeEach(() => {
  record = draftStores().record;
});

describe("record store", () => {
  it("adds, updates and removes transfer credit", () => {
    const id = record.addEntry({
      courseId: "21122",
      source: "transfer",
      units: 10,
      institution: "Pitt",
    });
    const general = record.addEntry({
      courseId: null,
      source: "transfer",
      units: 9,
      label: "Calc III at Pitt",
    });
    expect(record.entries.map((e) => e.id)).toEqual([id, general]);

    record.updateEntry(id, { term: "M26" });
    expect(record.entries[0]!.term).toBe("M26");

    record.removeEntry(general);
    expect(record.entries).toHaveLength(1);
  });

  it("removes every entry from one exam", () => {
    record.addEntries([
      { courseId: "21120", source: "exam", units: 10, exam: { id: "ap-calc-bc", score: "5" } },
      { courseId: "21122", source: "exam", units: 10, exam: { id: "ap-calc-bc", score: "5" } },
      { courseId: "15122", source: "taken", term: "F25", grade: "A" },
    ]);
    record.removeExam("ap-calc-bc");
    expect(record.entries.map((e) => e.courseId)).toEqual(["15122"]);
  });

  it("sets and clears overrides", () => {
    record.setOverride("15213", { prereqs: true, coreqs: false });
    expect(record.overrides["15213"]).toMatchObject({ prereqs: true });
    expect(record.overrides["15213"]!.coreqs).toBeUndefined();
    record.setOverride("15213", { prereqs: false });
    expect(record.overrides["15213"]).toBeUndefined();
  });
});

describe("recordTotals", () => {
  it("counts transfer and exam credit as earned, non-factorable units", () => {
    record.addEntries([
      { courseId: "15122", source: "taken", term: "F25", grade: "A" },
      { courseId: "21122", source: "transfer", institution: "Pitt" },
      { courseId: null, source: "transfer", units: 9, label: "Free elective" },
      { courseId: "21120", source: "exam", units: 10, exam: { id: "ap-calc-bc", score: "5" } },
      { courseId: "21120", source: "waiver", units: 0 },
    ]);
    const t = recordTotals(record.entries, unitsOf);
    expect(t.bySource).toEqual({ taken: 12, transfer: 19, exam: 10, waiver: 0 });
    expect(t.earned).toBe(41);
    expect(t.factorable).toBe(12);
    expect(t.nonFactorable).toBe(29);
    expect(t.qpa).toBe(4);
  });
});

describe("import and export", () => {
  it("round-trips transfer credit, including general units with no course", () => {
    record.addEntries([
      { courseId: "21122", source: "transfer", units: 10, institution: "Pitt", term: "M26" },
      { courseId: null, source: "transfer", units: 9, label: "Free elective" },
    ]);
    const parsed = parseRecordExport(JSON.parse(JSON.stringify(exportRecord(record.data))));
    expect("data" in parsed).toBe(true);
    if (!("data" in parsed)) return;
    expect(
      parsed.data.entries.map((e) => [e.courseId, e.units, e.label ?? null, e.term ?? null]),
    ).toEqual([
      ["21122", 10, null, "M26"],
      [null, 9, "Free elective", null],
    ]);

    record.clearRecord();
    record.importRecord(parsed.data, "replace");
    expect(record.entries).toHaveLength(2);
    record.importRecord(parsed.data, "merge");
    expect(record.entries).toHaveLength(2);
  });
});

describe("credit in requisite checks", () => {
  it("lets transfer credit satisfy prerequisites, ignoring minimum grades", () => {
    record.addEntry({ courseId: "15122", source: "transfer", units: 12 });
    const ctx = buildRequisiteContext({ record: record.data, catalog, term: "S27" });
    expect(checkCourse("15213", catalog.courses["15213"], ctx).level).toBe("ok");
  });

  it("counts transfer credit from a later term only from then on", () => {
    record.addEntry({ courseId: "21122", source: "transfer", units: 10, term: "M27" });
    const before = buildRequisiteContext({ record: record.data, catalog, term: "S27" });
    const after = buildRequisiteContext({ record: record.data, catalog, term: "F27" });
    expect(checkCourse("21259", catalog.courses["21259"], before).level).toBe("problem");
    expect(checkCourse("21259", catalog.courses["21259"], after).level).toBe("ok");
  });

  it("lets exam credit and placements satisfy prerequisites", () => {
    record.addEntries([{ courseId: "21122", source: "waiver", units: 0 }]);
    const ctx = buildRequisiteContext({ record: record.data, catalog, term: "S27" });
    expect(checkCourse("21259", catalog.courses["21259"], ctx).level).toBe("ok");
  });
});
