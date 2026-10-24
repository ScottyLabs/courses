// A Vite plugin that answers /api and /auth from the in-memory mock API (mock-server.ts), so the website can be
// developed before the Rust API is ready. Only `vite dev` uses it, and vite.config.ts loads this file only when
// MOCK_API=1, so it is never part of a build. See docs/src/frontend.md.
//
//   MOCK_API=1                       turn it on (`deno task dev:mock`)
//   MOCK_CATALOG_DIR=<dir>           Courseweave-format catalog files, e.g. cmu-course-planner/public/data
//   MOCK_SIGNED_IN=1                 start signed in (sign in and out with the website's buttons either way)
import type { IncomingMessage, ServerResponse } from "node:http";
import process from "node:process";
import type { Plugin } from "vite";
import { createMockServer } from "./mock-server.ts";

/** The request body as text (the mock API takes JSON only). */
async function readBody(req: IncomingMessage): Promise<string | undefined> {
  if (req.method === "GET" || req.method === "HEAD") return undefined;
  let body = "";
  req.setEncoding("utf8");
  for await (const chunk of req) body += String(chunk);
  return body === "" ? undefined : body;
}

async function send(res: ServerResponse, response: Response): Promise<void> {
  res.statusCode = response.status;
  response.headers.forEach((value, key) => {
    res.setHeader(key, value);
  });
  res.end(Buffer.from(await response.arrayBuffer()));
}

export function mockApi(): Plugin {
  return {
    name: "cmucourses-mock-api",
    apply: "serve",
    configureServer(server) {
      const mock = createMockServer({
        catalogDir: process.env.MOCK_CATALOG_DIR ?? null,
        signedIn: process.env.MOCK_SIGNED_IN === "1",
      });
      server.config.logger.info(
        `  Mock API on /api and /auth (${process.env.MOCK_CATALOG_DIR === undefined ? "no catalog: set MOCK_CATALOG_DIR" : `catalog from ${process.env.MOCK_CATALOG_DIR}`})`,
      );
      server.middlewares.use((req, res, next) => {
        const url = req.url ?? "/";
        if (!url.startsWith("/api/") && !url.startsWith("/auth/")) {
          next();
          return;
        }
        void (async () => {
          try {
            const body = await readBody(req);
            const headers = new Headers();
            for (const [key, value] of Object.entries(req.headers)) {
              if (typeof value === "string") headers.set(key, value);
            }
            const request = new Request(new URL(url, "http://localhost"), {
              method: req.method,
              headers,
              body,
            });
            await send(res, await mock.handle(request));
          } catch (e) {
            next(e);
          }
        })();
      });
    },
  };
}
