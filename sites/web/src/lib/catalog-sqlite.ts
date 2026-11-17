import { browser } from "$app/environment";
import publicCatalogUrl from "$lib/assets/public-browser.sqlite.gz?url&no-inline";
import { session } from "$lib/session";
import { get } from "svelte/store";
import type { BindingSpec, Database } from "@sqlite.org/sqlite-wasm";
import { createCatalogReader } from "./catalog-reader";

export type {
  CatalogCourse,
  CatalogInstructor,
  CatalogPage,
  CatalogPageOptions,
  CatalogTerm,
  CoursePageOptions,
  InstructorPageOptions,
} from "./catalog-reader";

const accessMessages = {
  "sign-in-required": "Sign in with CMU to view this catalog data.",
  "session-unconfirmed": "Your sign-in status could not be confirmed.",
  "cmu-access-required": "This catalog data requires verified CMU access.",
  "member-catalog-unavailable": "The CMU member catalog is unavailable.",
};

export class CatalogAccessError extends Error {
  constructor(readonly code: keyof typeof accessMessages) {
    super(accessMessages[code]);
    this.name = "CatalogAccessError";
  }
}

type MemberLoad = {
  request: AbortController;
  database: Promise<Database>;
};

let publicDatabase: Promise<Database> | undefined;
let memberLoad: MemberLoad | undefined;
let sqliteModule: ReturnType<typeof loadSqlite> | undefined;

function requireMemberAccess() {
  const current = get(session);
  if (current.status === "signed-out") throw new CatalogAccessError("sign-in-required");
  if (current.status !== "signed-in") throw new CatalogAccessError("session-unconfirmed");
  if (!current.canAccessMemberCatalog) throw new CatalogAccessError("cmu-access-required");
}

function requireBrowser() {
  if (!browser) throw new Error("The SQLite catalog is only available in the browser");
}

async function loadSqlite() {
  const { default: init } = await import("@sqlite.org/sqlite-wasm");
  return init();
}

async function openCatalog(member: boolean, signal?: AbortSignal): Promise<Database> {
  sqliteModule ??= loadSqlite().catch((error: unknown) => {
    sqliteModule = undefined;
    throw error;
  });
  const [sqlite, response] = await Promise.all([
    sqliteModule,
    fetch(member ? "/api/catalog/member-binary" : publicCatalogUrl, {
      credentials: member ? "same-origin" : "omit",
      cache: member ? "no-store" : "default",
      signal,
    }),
  ]);
  if (!response.ok) {
    if (member) {
      if (response.status === 401) throw new CatalogAccessError("sign-in-required");
      if (response.status === 403) throw new CatalogAccessError("cmu-access-required");
      throw new CatalogAccessError("member-catalog-unavailable");
    }
    throw new Error(`Catalog download failed (HTTP ${response.status})`);
  }

  // Fetch decodes the member response's Content-Encoding. The public asset is a gzip file.
  if (!response.body) throw new Error("Catalog response had no body");
  const decoded = member
    ? response
    : new Response(response.body.pipeThrough(new DecompressionStream("gzip")));
  const bytes = new Uint8Array(await decoded.arrayBuffer());
  signal?.throwIfAborted();
  const { capi, oo1, wasm } = sqlite;
  const db = new oo1.DB(":memory:");
  try {
    const data = wasm.allocFromTypedArray(bytes);
    // With FREEONCLOSE, SQLite also frees the buffer if deserialization fails.
    db.checkRc(
      capi.sqlite3_deserialize(
        db,
        "main",
        data,
        bytes.byteLength,
        bytes.byteLength,
        capi.SQLITE_DESERIALIZE_FREEONCLOSE | capi.SQLITE_DESERIALIZE_READONLY,
      ),
    );
    return db;
  } catch (error) {
    db.close();
    throw error;
  }
}

function publicCatalog() {
  publicDatabase ??= openCatalog(false).catch((error: unknown) => {
    publicDatabase = undefined;
    throw error;
  });
  return publicDatabase;
}

function memberCatalog(): MemberLoad {
  if (!memberLoad) {
    const request = new AbortController();
    const database = openCatalog(true, request.signal).catch((error: unknown) => {
      if (memberLoad === load) memberLoad = undefined;
      if (error instanceof CatalogAccessError || request.signal.aborted) throw error;
      throw new CatalogAccessError("member-catalog-unavailable");
    });
    const load = { request, database };
    memberLoad = load;
  }
  return memberLoad;
}

function clearMemberCatalog() {
  const previous = memberLoad;
  memberLoad = undefined;
  previous?.request.abort();
  void previous?.database.then(
    (db) => {
      db.close();
    },
    () => {},
  );
}

export function preloadCatalogs() {
  // A later query retries a failed preload.
  void publicCatalog().catch(() => {});
  const unsubscribe = session.subscribe((current) => {
    clearMemberCatalog();
    if (current.status === "signed-in" && current.canAccessMemberCatalog) {
      void memberCatalog().database.catch(() => {});
    }
  });
  window.addEventListener("pagehide", clearMemberCatalog);
  return () => {
    unsubscribe();
    window.removeEventListener("pagehide", clearMemberCatalog);
    clearMemberCatalog();
  };
}

/** Run a parameterized query against the public catalog. */
export async function queryCatalog(sql: string, bind?: BindingSpec) {
  requireBrowser();
  return (await publicCatalog()).selectObjects(sql, bind);
}

/** Run a parameterized query against the authorized CMU member catalog. */
export async function queryMemberCatalog(sql: string, bind?: BindingSpec) {
  requireBrowser();
  requireMemberAccess();
  const load = memberCatalog();
  try {
    const database = await load.database;
    load.request.signal.throwIfAborted();
    return database.selectObjects(sql, bind);
  } catch (error) {
    if (load.request.signal.aborted) {
      requireMemberAccess();
      throw new CatalogAccessError("session-unconfirmed");
    }
    throw error;
  }
}

/** Typed catalog reads for use by frontend screens. */
export const { listCourses, streamCourses, listInstructors, streamInstructors, listTerms } =
  createCatalogReader(queryCatalog, queryMemberCatalog);
