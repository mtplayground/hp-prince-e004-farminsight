# hp-prince-e004-farminsight

React SPA and Rust/Axum backend for a CSV analytics application.

## Project Layout

- `frontend/` contains the Vite, React, TypeScript, and Tailwind SPA.
- `backend/` contains the Axum server that exposes `/api/*` routes and serves the built SPA.

## Local Development

Install frontend dependencies:

```bash
npm install
```

Run the frontend development server:

```bash
npm run dev
```

Build the frontend:

```bash
npm run build
```

Run the backend host on `0.0.0.0:8080` after building the SPA:

```bash
export DATABASE_URL=$(cat /workspace/.database_url)
cargo run -p farminsight-backend
```

Runtime settings are read from environment variables. `DATABASE_URL` is required and must point to PostgreSQL. `HOST` defaults to `0.0.0.0`, `PORT` defaults to `8080`, `FRONTEND_DIST_DIR` defaults to `frontend/dist`, and `DATABASE_MAX_CONNECTIONS` defaults to `5`. Set `DATABASE_SSL_MODE` to `disable`, `prefer`, or `require` only when overriding the connection string's own SSL mode.

See `.env.example` for the complete backend environment contract, including PostgreSQL, object storage, auth service, legacy JWT fallback, and email proxy settings. The backend loads `.env` for local development when present.

## Self-hosted Deployment

For a bare self-hosted release without Docker or CI/CD, build a static frontend and release backend bundle:

```bash
npm ci
./scripts/build-self-hosted.sh
```

The bundle contains the Axum binary, `frontend/dist`, `backend/migrations`, and a runtime `.env.example`. See `docs/self-hosted.md` for the full build, environment, reverse-proxy, and systemd notes.
