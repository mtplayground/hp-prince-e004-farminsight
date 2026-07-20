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
cargo run -p farminsight-backend
```

Runtime settings are read from environment variables. `HOST` defaults to `0.0.0.0`, `PORT` defaults to `8080`, and `FRONTEND_DIST_DIR` defaults to `frontend/dist`.
