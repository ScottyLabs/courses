// What the stores share: their modes, their dependencies, and the copies of records they keep in sessionStorage.
// Signed out, a store's whole state is a draft under `cmucourses:draft:<name>`; signed in, whatever the server
// doesn't have yet is kept under `cmucourses:pending:<name>` with the account it belongs to.
import type { CellFactory } from "../data/cell.ts";
import { isObject, readJson, type StoreEnv } from "./env.ts";

/** A record as the server stores it. */
export interface Stored<T> {
  value: T;
  revision: number;
}

export type Edit<T> = (value: T) => T;

/** "in a minute", "in 5 minutes", "in a few hours": when a paused save tries again. */
export function retryIn(seconds: number): string {
  if (seconds >= 3600) return "in a few hours";
  if (seconds >= 120) return `in ${Math.round(seconds / 60)} minutes`;
  return "in a minute";
}

/** saved: the server has everything; saving: a write is pending or under way; offline: waiting for the network. */
export type SaveStatus = "saved" | "saving" | "error" | "offline";

const RANK: Record<SaveStatus, number> = { saved: 0, saving: 1, offline: 2, error: 3 };

/** The status to show for several records: the most pressing one. */
export function worstStatus(statuses: Iterable<SaveStatus>): SaveStatus {
  let worst: SaveStatus = "saved";
  for (const s of statuses) if (RANK[s] > RANK[worst]) worst = s;
  return worst;
}

/** draft: signed out, kept in this tab; loading: the account is loading; account: saved to the signed-in account. */
export type StoreMode = "draft" | "loading" | "account";

export interface StoreDeps {
  cell: CellFactory;
  env: StoreEnv;
  /** Quiet time before a write, in milliseconds. */
  debounceMs?: number;
}

export const DEFAULT_DEBOUNCE_MS = 1500;

export const SESSION_ENDED = "Your session ended. Sign in again to save your changes.";

export const draftKey = (name: string): string => `cmucourses:draft:${name}`;
export const pendingKey = (name: string): string => `cmucourses:pending:${name}`;

/** A record kept on this device, and the stored revision it's based on (null: never saved). */
export interface LocalCopy<T> {
  value: T;
  basedOn: number | null;
}

/** A stored record from untrusted JSON: the adapter's record plus a numeric revision. */
export function parseStored<T>(
  raw: unknown,
  normalize: (raw: unknown) => T | null,
): Stored<T> | null {
  if (!isObject(raw) || typeof raw.revision !== "number") return null;
  const value = normalize(raw);
  return value === null ? null : { value, revision: raw.revision };
}

/** A copy kept under `key`, or null; only `owner`'s when an owner is given. */
export function readCopy(
  env: StoreEnv,
  key: string,
  owner: string | null,
): Record<string, unknown> | null {
  const raw = readJson(env, key);
  if (!isObject(raw) || (owner !== null && raw.owner !== owner)) return null;
  return raw;
}

/** One record from a stored copy. */
export function localCopy<T>(
  entry: unknown,
  normalize: (raw: unknown) => T | null,
): LocalCopy<T> | null {
  if (!isObject(entry)) return null;
  const value = normalize(entry.value);
  return value === null
    ? null
    : { value, basedOn: typeof entry.basedOn === "number" ? entry.basedOn : null };
}

/** Records and deletions from a stored copy of a collection. */
export function readCollectionCopy<T>(
  env: StoreEnv,
  key: string,
  owner: string | null,
  normalize: (raw: unknown) => T | null,
): { items: LocalCopy<T>[]; deleted: string[] } {
  const raw = readCopy(env, key, owner);
  const items: LocalCopy<T>[] = [];
  const deleted: string[] = [];
  if (raw === null) return { items, deleted };
  for (const entry of Array.isArray(raw.items) ? raw.items : []) {
    const copy = localCopy(entry, normalize);
    if (copy !== null) items.push(copy);
  }
  for (const d of Array.isArray(raw.deleted) ? raw.deleted : []) {
    if (isObject(d) && typeof d.id === "string") deleted.push(d.id);
  }
  return { items, deleted };
}
