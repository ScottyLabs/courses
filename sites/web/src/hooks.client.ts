import type { HandleClientError } from "@sveltejs/kit";
import { handleStaleBuild } from "$lib/data";

// A page whose code was removed by a newer deploy reloads the tab once instead of showing an error.
export const handleError: HandleClientError = ({ error }) => {
  handleStaleBuild(error);
};
