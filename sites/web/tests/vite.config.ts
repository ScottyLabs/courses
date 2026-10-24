import { defineConfig } from "vite";
import { fileURLToPath } from "node:url";
import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
// Test-only server: serves ONLY the sanitized public catalog, never personal/member data.
export default defineConfig({
 root: fileURLToPath(new URL("..", import.meta.url)),
 plugins: [{ name: "public-catalog-fixture", configureServer(server) {
   const bytes=readFileSync(fileURLToPath(new URL("../../../data/catalog/public-browser.sqlite",import.meta.url)));
   const hash=createHash("sha256").update(bytes).digest("hex");
   server.middlewares.use((req,res,next) => {
     if(req.url === "/api/catalog/version") {res.setHeader("Content-Type","application/json");res.end(JSON.stringify({hash}));}
     else if(req.url === "/api/catalog/binary") {res.setHeader("Content-Type","application/vnd.sqlite3");res.end(bytes);}
     else next();
   });
 }}],
});
