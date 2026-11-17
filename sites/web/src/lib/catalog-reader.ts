type Query = (sql: string, bind: (string | number)[]) => Promise<unknown[]>;

export type CatalogPageOptions = {
  /** One-based page number. */
  page?: number;
  /** Defaults to 25; accepts 1 to 100. */
  pageSize?: number;
  signal?: AbortSignal;
};

export type CatalogPage<Row> = {
  items: Row[];
  page: number;
  nextPage: number | null;
};

type Sort<Field> = { field: Field; direction?: "asc" | "desc" };
type Audience = "public" | "member";
type CourseSort = "code" | "name" | "department" | "level" | "units";

type CourseFilters = {
  /** Omit to browse all courses, with each code appearing once. */
  term?: string;
  search?: string;
  /** Department name or course code prefix, e.g. "15". */
  department?: string;
  level?: number;
  minLevel?: number;
  units?: number;
  offeredIn?: "fall" | "spring" | "summer";
};

export type CoursePageOptions = CatalogPageOptions &
  CourseFilters &
  (
    | { audience?: "public"; sort?: Sort<CourseSort>; minFce?: never }
    | {
        audience: "member";
        sort?: Sort<CourseSort | "overallCourse">;
        minFce?: number;
      }
  );

export type InstructorPageOptions = CatalogPageOptions & {
  search?: string;
  department?: string;
  sort?: Sort<"name" | "department" | "id">;
};

export type CatalogCourse = {
  code: string;
  name: string;
  description: string;
  department: string;
  level: number;
  units: number;
  /** Overall FCE score; null in the public catalog or when unavailable. */
  overallCourse: number | null;
};

export type CatalogInstructor = {
  id: string;
  name: string;
  department: string;
};
export type CatalogTerm = { id: string; label: string; hasCourses: boolean };

const courseSorts = {
  code: "code",
  name: "name COLLATE NOCASE",
  department: "department COLLATE NOCASE",
  level: "level",
  units: "units",
};

const memberCourseSorts = { ...courseSorts, overallCourse: "overall_course" };

const instructorSorts = {
  id: "id",
  name: "name COLLATE NOCASE",
  department: "department COLLATE NOCASE",
};

function orderBy(columns: Record<string, string>, sort: Sort<string>, tieBreaker: string) {
  if (!Object.hasOwn(columns, sort.field)) {
    throw new TypeError("Unknown catalog sort field");
  }
  const direction = sort.direction ?? "asc";
  if (direction !== "asc" && direction !== "desc") {
    throw new TypeError("Unknown catalog sort direction");
  }
  return `${columns[sort.field]} ${
    direction === "asc" ? "ASC" : "DESC"
  } NULLS LAST, ${tieBreaker} ASC`;
}

function pager<Row>(
  query: Query,
  sql: string,
  params: (string | number)[],
  { page = 1, pageSize = 25, signal }: CatalogPageOptions,
) {
  if (!Number.isSafeInteger(pageSize) || pageSize < 1 || pageSize > 100) {
    throw new RangeError("Catalog page size must be an integer from 1 to 100");
  }

  async function read(current: number): Promise<CatalogPage<Row>> {
    signal?.throwIfAborted();
    const offset = (current - 1) * pageSize;
    if (!Number.isSafeInteger(current) || current < 1 || !Number.isSafeInteger(offset)) {
      throw new RangeError("Invalid catalog page number");
    }
    // eslint-disable-next-line typescript/no-unsafe-type-assertion -- Each caller defines the SQL projection for Row.
    const rows = (await query(`${sql} LIMIT ? OFFSET ?`, [
      ...params,
      pageSize + 1,
      offset,
    ])) as Row[];
    signal?.throwIfAborted();
    return {
      items: rows.slice(0, pageSize),
      page: current,
      nextPage: rows.length > pageSize ? current + 1 : null,
    };
  }

  return {
    page: () => read(page),
    async *stream(): AsyncGenerator<CatalogPage<Row>, void> {
      let next: number | null = page;
      while (next !== null) {
        // eslint-disable-next-line no-await-in-loop -- Request each page only when the consumer advances.
        const result = await read(next);
        if (result.items.length === 0) return;
        yield result;
        next = result.nextPage;
      }
    },
  };
}

function courseFilters(options: CoursePageOptions) {
  const { minFce } = options;

  const clauses = ["term_id = ?"];
  const params: (string | number)[] = [options.term ?? ""];
  const add = (clause: string, ...values: (string | number)[]) => {
    clauses.push(clause);
    params.push(...values);
  };

  const search = options.search?.trim().toLowerCase() ?? "";
  if (search) {
    const match =
      "instr(lower(code || ' ' || coalesce(name, '') || ' ' || coalesce(description, '')), ?) > 0";
    const compact = search.replaceAll("-", "");
    if (/^\d+$/u.test(compact)) {
      add(`(${match} OR instr(replace(code, '-', ''), ?) > 0)`, search, compact);
    } else {
      add(match, search);
    }
  }

  const department = options.department?.trim() ?? "";
  if (department) {
    add("(department = ? COLLATE NOCASE OR instr(code, ?) = 1)", department, department);
  }
  for (const [clause, value] of [
    ["level = ?", options.level],
    ["level >= ?", options.minLevel],
    ["units = ?", options.units],
    ["overall_course >= ?", minFce],
  ] as const) {
    if (value === undefined) continue;
    if (!Number.isFinite(value)) {
      throw new TypeError("Catalog numeric filters must be finite");
    }
    add(clause, value);
  }
  if (options.offeredIn) {
    add("instr('|' || offered_in || '|', '|' || ? || '|') > 0", options.offeredIn);
  }

  return { clauses, params };
}

function courses(queryPublic: Query, queryMember: Query, options: CoursePageOptions) {
  const { audience, minFce, sort } = options;
  const member = audience === "member";
  if (!member && minFce !== undefined) {
    throw new TypeError("FCE filters require the member catalog");
  }
  const { clauses, params } = courseFilters(options);
  return pager<CatalogCourse>(
    member ? queryMember : queryPublic,
    `SELECT code, name, description, department, level, units,
            ${member ? "overall_course" : "NULL"} AS overallCourse
       FROM catalog_courses WHERE ${clauses.join(" AND ")}
       ORDER BY ${orderBy(
         member ? memberCourseSorts : courseSorts,
         sort ?? { field: "code" },
         "code",
       )}`,
    params,
    options,
  );
}

function instructors(queryMember: Query, options: InstructorPageOptions) {
  const clauses = [];
  const params = [];
  const search = options.search?.trim().toLowerCase() ?? "";
  if (search) {
    clauses.push("instr(lower(name || ' ' || coalesce(department, '')), ?) > 0");
    params.push(search);
  }
  const department = options.department?.trim() ?? "";
  if (department) {
    clauses.push("department = ? COLLATE NOCASE");
    params.push(department);
  }

  return pager<CatalogInstructor>(
    queryMember,
    `SELECT id, name, coalesce(department, '') AS department FROM catalog_instructors
       ${clauses.length > 0 ? `WHERE ${clauses.join(" AND ")}` : ""}
       ORDER BY ${orderBy(instructorSorts, options.sort ?? { field: "name" }, "id")}`,
    params,
    options,
  );
}

export function createCatalogReader(queryPublic: Query, queryMember: Query) {
  return {
    listCourses: async (options: CoursePageOptions = {}) => {
      const page = await courses(queryPublic, queryMember, options).page();
      return page;
    },
    streamCourses: (options: CoursePageOptions = {}) =>
      courses(queryPublic, queryMember, options).stream(),
    listInstructors: async (options: InstructorPageOptions = {}) => {
      const page = await instructors(queryMember, options).page();
      return page;
    },
    streamInstructors: (options: InstructorPageOptions = {}) =>
      instructors(queryMember, options).stream(),
    listTerms: async ({
      audience = "public",
      signal,
    }: {
      audience?: Audience;
      signal?: AbortSignal;
    } = {}): Promise<CatalogTerm[]> => {
      signal?.throwIfAborted();
      const query = audience === "member" ? queryMember : queryPublic;
      // eslint-disable-next-line typescript/no-unsafe-type-assertion -- The SELECT defines these column names and types.
      const rows = (await query(
        `SELECT t.id, json_extract(t.payload_json, '$.label') AS label,
                EXISTS (SELECT 1 FROM catalog_coverage
                         WHERE dataset = 'courses' AND term_id = t.id) AS hasCourses
           FROM catalog_terms t ORDER BY t.sort_order, t.id`,
        [],
      )) as (Omit<CatalogTerm, "hasCourses"> & { hasCourses: number })[];
      signal?.throwIfAborted();
      return rows.map(({ id, label, hasCourses }) => ({ id, label, hasCourses: hasCourses === 1 }));
    },
  };
}
