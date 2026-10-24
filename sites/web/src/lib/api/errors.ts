// What a failed API request tells the caller: the `{ code, message }` body as an ApiError, plus a 429's Retry-After
// and a 409's stored record.
import type { ErrorBody, ErrorCode } from "./types.ts";

/** Longest wait a 429's Retry-After is taken at, in seconds. */
const MAX_RETRY_AFTER = 6 * 60 * 60;

/** What a failed request tells the caller. */
export class ApiError extends Error {
  override readonly name = "ApiError";
  /** HTTP status; 0 when no response arrived. */
  readonly status: number;
  /** The body's code; "network" when no response arrived. */
  readonly code: ErrorCode | "network";
  /** Seconds to wait before trying again, on 429. */
  readonly retryAfter: number | null;
  /** The stored record, on 409. */
  readonly current: unknown;

  constructor(
    status: number,
    code: ErrorCode | "network",
    message: string,
    extra: { retryAfter?: number | null; current?: unknown } = {},
  ) {
    super(message);
    this.status = status;
    this.code = code;
    this.retryAfter = extra.retryAfter ?? null;
    this.current = extra.current;
  }

  /** Not signed in, or the session ended. */
  get signInRequired(): boolean {
    return this.status === 401;
  }

  /** A write based on a stale revision; `current` holds the stored record. */
  get conflict(): boolean {
    return this.status === 409;
  }

  /** Too many requests; try again after `retryAfter` seconds. */
  get rateLimited(): boolean {
    return this.status === 429;
  }

  /** The request never got an answer (offline, or the server is unreachable). */
  get offline(): boolean {
    return this.code === "network";
  }
}

const ERROR_CODES = new Set<string>([
  "auth",
  "forbidden",
  "not_found",
  "conflict",
  "too_large",
  "too_many",
  "bad_request",
  "unavailable",
]);

export function isErrorBody(v: unknown): v is ErrorBody {
  if (typeof v !== "object" || v === null) return false;
  const code: unknown = Reflect.get(v, "code");
  return (
    typeof code === "string" &&
    ERROR_CODES.has(code) &&
    typeof Reflect.get(v, "message") === "string"
  );
}

/** The code a status implies when the body doesn't say. */
export function codeForStatus(status: number): ErrorCode {
  switch (status) {
    case 400:
      return "bad_request";
    case 401:
      return "auth";
    case 403:
      return "forbidden";
    case 404:
      return "not_found";
    case 409:
      return "conflict";
    case 413:
      return "too_large";
    case 429:
      return "too_many";
    default:
      return "unavailable";
  }
}

/** Retry-After in seconds: a number of seconds or an HTTP date; a minute when missing or unreadable. */
export function retryAfterSeconds(value: string | null, now = Date.now()): number {
  if (value === null || value.trim() === "") return 60;
  const seconds = Number(value);
  if (Number.isFinite(seconds) && seconds >= 0)
    return Math.min(Math.max(seconds, 1), MAX_RETRY_AFTER);
  const at = Date.parse(value);
  if (Number.isNaN(at)) return 60;
  return Math.min(Math.max(Math.ceil((at - now) / 1000), 1), MAX_RETRY_AFTER);
}
