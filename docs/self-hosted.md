# Self-hosted Deployment

Farminsight can run as one Rust/Axum process that serves both `/api/*` and the built React SPA. This path is intentionally bare: no Docker image and no CI/CD workflow.

## Build

Prerequisites:

- Node.js 20 or newer
- Rust toolchain with Cargo
- PostgreSQL 16 or compatible
- S3-compatible object storage for uploaded CSV files

From the repository root:

```bash
npm ci
./scripts/build-self-hosted.sh
```

The script writes a bundle to `build/self-hosted` by default:

- `bin/farminsight-backend`
- `frontend/dist`
- `backend/migrations`
- `.env.example`
- `README.txt`

Set `SELF_HOSTED_OUT_DIR=/opt/farminsight` to write the bundle somewhere else.

## Configure

Copy the bundled `.env.example` to `.env` and set real values. Required groups:

- `DATABASE_URL`: PostgreSQL connection string. The app uses PostgreSQL for all persistent state and runs migrations on startup.
- `OBJECT_STORAGE_*`: S3-compatible storage for committed CSV files. Set `OBJECT_STORAGE_PREFIX` so objects are isolated from other apps in the bucket.
- `MCTAI_AUTH_*`: Ideavibes auth service. The app verifies the `mctai_session` cookie and does not issue its own app JWT.

Recommended runtime paths when launching from the bundle directory:

```bash
HOST=0.0.0.0
PORT=8080
FRONTEND_DIST_DIR=frontend/dist
MIGRATIONS_DIR=backend/migrations
SELF_URL=https://farminsight.example.com
ALLOWED_CORS_ORIGIN=https://farminsight.example.com
```

If you launch from another working directory, set `FRONTEND_DIST_DIR` and `MIGRATIONS_DIR` to absolute paths.

## Run

```bash
cd build/self-hosted
cp .env.example .env
# edit .env
set -a
. ./.env
set +a
./bin/farminsight-backend
```

The process listens on `HOST:PORT`. Put TLS and any public hostname in front of it with a reverse proxy such as nginx, Caddy, or a load balancer. The proxy should forward normal HTTP requests to the Axum process; no special route split is needed because the same process serves `/api/*` and the SPA fallback.

## Health Check

Use:

```bash
curl -f http://127.0.0.1:8080/api/health
```

A healthy response reports `status: "ok"` and `database: "ok"`.

## systemd Example

```ini
[Unit]
Description=Farminsight
After=network-online.target
Wants=network-online.target

[Service]
WorkingDirectory=/opt/farminsight
EnvironmentFile=/opt/farminsight/.env
ExecStart=/opt/farminsight/bin/farminsight-backend
Restart=always
RestartSec=5
User=farminsight
Group=farminsight

[Install]
WantedBy=multi-user.target
```

Reload systemd after installing the unit:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now farminsight
sudo systemctl status farminsight
```
