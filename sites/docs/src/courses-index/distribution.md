# Catalog distribution

`courses-index` builds full public and member SQLite catalogs from Stellic exports for API queries and smaller gzip snapshots for the browser. The browser snapshots cover default course rows and the three latest terms.

The Forgejo scrape workflow builds and verifies the catalogs, uploads six encrypted artifacts to an immutable release prefix, and attaches the release manifest. Commit the manifest as `deployments/catalog.json` to select that release for the next Kennel deployment. [Deployment runbook](../deployment.md)

`courses-web-api` installs the pinned release at startup. The public browser fetch omits credentials at `/api/catalog/binary`. The member browser fetch uses the session cookie at `/api/catalog/member-binary`. Both responses declare `Content-Encoding: gzip`, and the browser decodes them before opening SQLite.
