# Maintainer guide

Start with [backend development](backend-development.md) for the current skeleton, file map, endpoint inventory and implementation sequence. The frontend and database infrastructure remain; product endpoint behavior has intentionally been removed.

Use the root README for development startup, [database requirements](api/database-requirements.md) for storage constraints, and [frontend contract](api/frontend-contract.md) for the integration target. Older design and deployment documents retain context, but do not establish current feature availability.

Keep public/member data separate, enforce ownership in SQL, preserve existing personal records with versioned migrations, and add feature tests as implementations return. Authentication infrastructure remains active; profile registration and all other feature persistence must be implemented explicitly.
