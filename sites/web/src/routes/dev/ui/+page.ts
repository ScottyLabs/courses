import { error } from "@sveltejs/kit";
import type { PageLoad } from "./$types";

// The component gallery exists only in development builds.
export const load: PageLoad = () => {
  if (!import.meta.env.DEV) error(404, "Not found");
};
