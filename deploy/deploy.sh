#!/usr/bin/env bash
# Usage: ./deploy/deploy.sh user@YOUR_SERVER_IP
set -e

SERVER=${1:?Usage: $0 user@server_ip}

echo "==> Building frontend..."
cd frontend
PUBLIC_API_URL="" npm run build
cd ..

echo "==> Uploading frontend to server..."
rsync -avz --delete frontend/dist/ "$SERVER:/var/www/incopter/"

echo "==> Uploading backend source to server..."
rsync -avz --delete --exclude target backend/ "$SERVER:/opt/incopter-backend/src_upload/"

echo "==> Building & restarting backend on server..."
ssh "$SERVER" bash <<'REMOTE'
  set -e
  source "$HOME/.cargo/env"
  cd /opt/incopter-backend/src_upload
  cargo build --release
  cp target/release/incopter-backend /opt/incopter-backend/incopter-backend
  systemctl restart incopter-backend
  echo "Backend restarted OK"
REMOTE

echo "==> Done! Site live at https://incopter.gr"
