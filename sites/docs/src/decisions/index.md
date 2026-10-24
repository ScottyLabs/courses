# Architecture decisions

| # | Decision | Status |
|---|---|---|
| 1 | [Record architecture decisions](0001-record-architecture-decisions.md) | accepted |
| 2 | [Serve the website and a public course API](0002-serve-the-website-and-a-public-course-api.md) | accepted |
| 3 | [Query the catalog in the browser](0003-query-the-catalog-in-the-browser.md) | accepted |
| 4 | [Publish the catalog to object storage](0004-publish-the-catalog-to-object-storage.md) | accepted |
| 5 | [Use SQLite for the catalog](0005-use-sqlite-for-the-catalog.md) | superseded by 13 |
| 6 | [Use Diesel for catalog queries](0006-use-diesel-for-catalog-queries.md) | browser queries superseded by 13 |
| 7 | [Load the catalog into memory in the browser](0007-load-the-catalog-into-memory-in-the-browser.md) | superseded by 16 |
| 8 | [Cache the catalog in OPFS](0008-cache-the-catalog-in-opfs.md) | amended by 16 |
| 9 | [Share one in-memory catalog across the public course API's connections](0009-share-one-in-memory-catalog-across-api-connections.md) | accepted |
| 10 | [Build the catalog separately from scraping](0010-build-the-catalog-separately-from-scraping.md) | amended by 15 |
| 11 | [Version raw scrape output by format](0011-version-raw-scrape-output-by-format.md) | accepted |
| 12 | [Delete catalogs no deployed branch uses](0012-delete-catalogs-no-deployed-branch-uses.md) | accepted |
| 13 | [Use API-first DuckDB browser catalog reads](0013-use-api-first-duckdb-browser-reads.md) | superseded by 16 |
| 14 | [Bring Courseweave's features into CMU Courses](0014-bring-courseweave-features-into-cmu-courses.md) | accepted |
| 15 | [Build the catalog in Rust, as one SQLite file](0015-build-the-catalog-in-rust.md) | accepted |
| 16 | [Serve the catalog to the browser as JSON documents](0016-serve-the-catalog-as-json-documents.md) | accepted |
| 17 | [Store personal data on the server](0017-store-personal-data-on-the-server.md) | accepted |
| 21 | [Sort catalog strings with Unicode collation](0021-sort-catalog-strings-with-unicode-collation.md) | accepted |
| 22 | [Leave build times out of FCE and program documents](0022-leave-build-times-out-of-fce-and-program-documents.md) | accepted |
| 30 | [Build UI primitives on native elements and the top layer](0030-build-ui-primitives-on-native-elements.md) | accepted |
| 31 | [Scroll the page, with a sticky header and rail](0031-scroll-the-page-with-a-sticky-rail.md) | accepted |
