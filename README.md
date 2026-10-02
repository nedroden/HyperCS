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

- Web: http://localhost:5173
- API: http://localhost:8080/api/v1/health

## Development

```bash
./lint.sh              # clippy, oxlint and the architecture linter
./format.sh            # rustfmt and Prettier (use --check to only verify)
```

See [`CLAUDE.md`](CLAUDE.md) for the architecture and conventions.
