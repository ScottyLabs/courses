// The web API client: `api` for the website's own requests, createApi() for tests and other origins.
import { ApiClient } from "./client.ts";
import { createApi } from "./endpoints.ts";

export { ApiClient, ApiError, retryAfterSeconds } from "./client.ts";
export type { ClientOptions, FetchLike, RequestOptions } from "./client.ts";
export { catalogPath, createApi, syllabusPath } from "./endpoints.ts";
export type { Api } from "./endpoints.ts";
export type * from "./types.ts";

/** The website's client: same origin, the browser's fetch. */
export const api = createApi(new ApiClient());
