#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")"

echo "==> Building release binary..."
(cd app && cargo build --release)

echo "==> Copying binary to x10..."
scp app/target/release/app x10:/home/CryptoWrap/app/release-cw.new

echo "==> Replacing binary atomically and restarting container..."
ssh x10 "
  mv /home/CryptoWrap/app/release-cw.new /home/CryptoWrap/app/release-cw &&
  docker compose -f /home/CryptoWrap/compose.yaml restart main-cv
"

echo "==> Done"
