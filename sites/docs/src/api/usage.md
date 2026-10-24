# Course data for developers

Student developers can use CMU Courses' public course data in their own projects in two ways.

- **The public course API** in `courses-web-api`: `GET /api/terms`, `GET /api/courses` (search with filters, paginated)
  and `GET /api/courses/{code}`. It answers from the catalog's relational tables, which hold public fields only. See
  [the web API](web-api.md#public-course-api) for parameters and responses, and `/api/scalar` on a running server for
  the OpenAPI reference.
- **Catalog documents**: `GET /api/catalog/<name>` serves the same JSON the website loads, such as `terms` or
  `term/S27`. Signed-out callers get the public versions, which leave out instructors, meeting rooms, FCEs, syllabi
  and degree programs ([decision 16](../decisions/0016-serve-the-catalog-as-json-documents.md)). The document list is
  in [Courseweave features in CMU Courses](../courseweave.md#catalog-documents).

FCEs, instructors, syllabi and Stellic degree programs are for signed-in CMU students only, as on the website, and
aren't available to other applications.

The older standalone `courses-api` service (`api.cmu.courses`) predates this catalog and serves its own format.
