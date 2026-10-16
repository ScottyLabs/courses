# Architecture decisions

| # | Decision | Status |
|---|---|---|
| 1 | [Record architecture decisions](0001-record-architecture-decisions.md) | accepted |
| 2 | [Serve the website and a public course API](0002-serve-the-website-and-a-public-course-api.md) | accepted |
| 3 | [Query the catalog in the browser](0003-query-the-catalog-in-the-browser.md) | accepted |
| 4 | [Publish the catalog to object storage](0004-publish-the-catalog-to-object-storage.md) | accepted |
| 5 | [Use SQLite for the catalog](0005-use-sqlite-for-the-catalog.md) | superseded by 13 |
| 6 | [Use Diesel for catalog queries](0006-use-diesel-for-catalog-queries.md) | browser queries superseded by 13 |
| 7 | [Load the catalog into memory in the browser](0007-load-the-catalog-into-memory-in-the-browser.md) | mechanism superseded by 13 |
| 8 | [Cache the catalog in OPFS](0008-cache-the-catalog-in-opfs.md) | accepted |
| 9 | [Share one in-memory catalog across the public course API's connections](0009-share-one-in-memory-catalog-across-api-connections.md) | accepted |
| 10 | [Build the catalog separately from scraping](0010-build-the-catalog-separately-from-scraping.md) | accepted |
| 11 | [Version raw scrape output by format](0011-version-raw-scrape-output-by-format.md) | accepted |
| 12 | [Delete catalogs no deployed branch uses](0012-delete-catalogs-no-deployed-branch-uses.md) | accepted |
| 13 | [Use API-first DuckDB browser catalog reads](0013-use-api-first-duckdb-browser-reads.md) | accepted |
