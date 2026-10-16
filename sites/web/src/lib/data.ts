import { api } from "./contracts";
import { createDataClient } from "./data-client";

export const data = createDataClient(api, async (user, signal) => {
  // The cache, decoder, and query engine are a separate lazy-loaded chunk.
  const { openLocalCatalog } = await import("./local-catalog");
  return openLocalCatalog(user, signal);
});

export async function clearLocalCatalog() {
  data.resetLocalCatalog();
  const { clearCatalogCache } = await import("./catalog");
  await clearCatalogCache();
}
