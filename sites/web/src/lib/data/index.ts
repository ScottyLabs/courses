// Catalog documents and the signed-in session (decision 16): `catalog` and `session` for the website, the classes
// and helpers for tests and other wiring. See docs/src/frontend.md.
export { catalog, session } from "./instances.svelte.ts";
export { Catalog } from "./catalog.ts";
export { SignInRequiredError, settle } from "./errors.ts";
export type { Loaded } from "./errors.ts";
export { Session, browserEnvironment, safeReturnPath, signInUrl } from "./session.ts";
export type { AuthError, SessionEnvironment, SessionStatus } from "./session.ts";
export { docs, isMemberOnly, prefixOf } from "./documents.ts";
export type { BuildInfo, DocumentTypes, RegistrarFinal } from "./documents.ts";
export { resource } from "./resource.svelte.ts";
export type { Resource, ResourceStatus } from "./resource.svelte.ts";
export {
  handleStaleBuild,
  installUpdateReload,
  isStaleBuildError,
  reloadForUpdate,
} from "./app-update.ts";
export { plainCell } from "./cell.ts";
export type { Cell, CellFactory } from "./cell.ts";
export { runeCell } from "./cell.svelte.ts";
