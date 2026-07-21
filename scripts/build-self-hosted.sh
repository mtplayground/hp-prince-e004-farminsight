#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${SELF_HOSTED_OUT_DIR:-"$ROOT_DIR/build/self-hosted"}"

cd "$ROOT_DIR"

npm run build
cargo build --release -p farminsight-backend

rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR/bin" "$OUT_DIR/frontend" "$OUT_DIR/backend"

cp "$ROOT_DIR/target/release/farminsight-backend" "$OUT_DIR/bin/farminsight-backend"
cp -R "$ROOT_DIR/frontend/dist" "$OUT_DIR/frontend/dist"
cp -R "$ROOT_DIR/backend/migrations" "$OUT_DIR/backend/migrations"
cp "$ROOT_DIR/deploy/self-hosted.env.example" "$OUT_DIR/.env.example"

cat > "$OUT_DIR/README.txt" <<'README'
Farminsight self-hosted bundle

Run from this directory so the default relative paths resolve:

  cp .env.example .env
  # edit .env with real PostgreSQL, object storage, auth, and email values
  set -a
  . ./.env
  set +a
  ./bin/farminsight-backend

The server listens on HOST:PORT and serves frontend/dist with SPA fallback.
PostgreSQL migrations are loaded from backend/migrations.
README

echo "Self-hosted bundle written to $OUT_DIR"
