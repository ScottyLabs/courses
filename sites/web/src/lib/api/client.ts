// A same-origin JSON client for the web API. Errors are `{ code, message }` bodies, thrown as ApiError (errors.ts); a
// request that gets no answer at all is an ApiError with code "network", so callers can tell "offline" from "the
// server said no".
import { ApiError, codeForStatus, isErrorBody, retryAfterSeconds } from "./errors.ts";

export { ApiError, retryAfterSeconds } from "./errors.ts";

export type FetchLike = (input: string, init?: RequestInit) => Promise<Response>;

function isJson(r: Response): boolean {
  return (r.headers.get("content-type") ?? "").includes("json");
}

export interface RequestOptions {
  /** Sent as JSON. */
  body?: unknown;
  /** Sent as multipart form data instead of `body`. */
  form?: FormData;
  query?: Record<string, string | number | boolean | null | undefined>;
  signal?: AbortSignal;
}

export interface ClientOptions {
  /** Defaults to the global fetch. */
  fetch?: FetchLike;
  /** Prefix for every path; "" for the same origin. */
  base?: string;
}

/** Sends requests to the API and turns failures into ApiError. */
export class ApiClient {
  readonly #fetch: FetchLike;
  readonly #base: string;
  #unauthorized: ((path: string) => void) | null = null;

  constructor(options: ClientOptions = {}) {
    this.#fetch = options.fetch ?? ((input, init) => fetch(input, init));
    this.#base = options.base ?? "";
  }

  /** Called on every 401, with the path asked for (the session uses it to notice an expired sign-in). */
  onUnauthorized(handler: ((path: string) => void) | null): void {
    this.#unauthorized = handler;
  }

  /** The URL for a path, with its query string. */
  url(path: string, query?: RequestOptions["query"]): string {
    const params = new URLSearchParams();
    for (const [k, v] of Object.entries(query ?? {})) {
      if (v !== null && v !== undefined) params.set(k, String(v));
    }
    const qs = params.toString();
    return `${this.#base}${path}${qs === "" ? "" : `?${qs}`}`;
  }

  async request<T>(method: string, path: string, options: RequestOptions = {}): Promise<T> {
    const headers: Record<string, string> = { Accept: "application/json" };
    let body: BodyInit | undefined;
    if (options.form !== undefined) body = options.form;
    else if (options.body !== undefined) {
      headers["Content-Type"] = "application/json";
      body = JSON.stringify(options.body);
    }
    let response: Response;
    try {
      response = await this.#fetch(this.url(path, options.query), {
        method,
        headers,
        body,
        credentials: "same-origin",
        signal: options.signal,
      });
    } catch (error) {
      if (options.signal?.aborted === true) throw error;
      throw new ApiError(
        0,
        "network",
        "Couldn't reach the server. Check your connection and try again.",
      );
    }
    if (response.ok) return readBody<T>(response);
    throw await this.#failure(path, response);
  }

  get<T>(path: string, options?: Omit<RequestOptions, "body" | "form">): Promise<T> {
    return this.request<T>("GET", path, options);
  }

  post<T>(path: string, options?: RequestOptions): Promise<T> {
    return this.request<T>("POST", path, options);
  }

  put<T>(path: string, options?: RequestOptions): Promise<T> {
    return this.request<T>("PUT", path, options);
  }

  delete<T>(path: string, options?: RequestOptions): Promise<T> {
    return this.request<T>("DELETE", path, options);
  }

  async #failure(path: string, response: Response): Promise<ApiError> {
    let parsed: unknown = null;
    if (isJson(response)) {
      try {
        parsed = await response.json();
      } catch {
        parsed = null;
      }
    }
    const body = isErrorBody(parsed) ? parsed : null;
    const code = body?.code ?? codeForStatus(response.status);
    const message =
      body?.message ??
      (response.status === 401
        ? "Sign in with your CMU account."
        : `The server answered ${response.status}.`);
    const retryAfter =
      response.status === 429 ? retryAfterSeconds(response.headers.get("Retry-After")) : null;
    if (response.status === 401) this.#unauthorized?.(path);
    return new ApiError(response.status, code, message, { retryAfter, current: body?.current });
  }
}

async function readBody<T>(response: Response): Promise<T> {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- 204 has no body; callers asking for one get void
  if (response.status === 204) return undefined as T;
  // A development server answers unknown paths with its HTML page; that isn't the API.
  if (!isJson(response))
    throw new ApiError(response.status, "unavailable", "The API isn't available here.");
  try {
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the API's documented response for this route
    return (await response.json()) as T;
  } catch {
    throw new ApiError(response.status, "unavailable", "The server sent an unreadable answer.");
  }
}
