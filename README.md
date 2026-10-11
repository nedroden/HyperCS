<p align="center">
  <img src="web/public/logo.svg" alt="HyperCS" height="72">
</p>

<p align="center">
  Open-source community software: a CMS and forum software in one, started as a Hacktoberfest project.
</p>

## Stack

- **API:** Rust (axum, sqlx, PostgreSQL) in [`api/`](api/)
- **Web:** React + TypeScript (Vite, Feature-Sliced Design) in [`web/`](web/)
- **Design reference:** static HTML mockups in [`mocks/`](mocks/), requirements in [`planning/`](planning/requirements.md)

## Getting started

Start the whole stack (PostgreSQL, API and web dev server) with Docker Compose:

```bash
docker compose up --build
```

(With the standalone binary, use `docker-compose up --build`.)

- Web: http://localhost:5173
- API: http://localhost:8080/api/v1/health, Swagger UI: http://localhost:8080/api/v1/docs
- First admin: `ADMIN_USERNAME` / `ADMIN_PASSWORD` from `docker-compose.yml` (local development only; change them for anything else).

The initial migration is edited in place while the project is pre-release. If you have an old database volume and the API fails on a migration mismatch, reset it with `docker compose down -v`.

## Development

```bash
./lint.sh              # clippy, oxlint and the architecture linter
./format.sh            # rustfmt and Prettier (use --check to only verify)
(cd api && cargo test --workspace)   # integration tests need Docker (Testcontainers)
(cd web && npm test)
(cd web && npm run generate:api)     # regenerate the typed client from the OpenAPI document
(cd web && npm run check:api)        # fail if the generated client is out of date
```

With Colima, run the API tests with `DOCKER_HOST=unix://$HOME/.colima/default/docker.sock TESTCONTAINERS_RYUK_DISABLED=true`.

See [`CLAUDE.md`](CLAUDE.md) for the architecture and conventions.
