#!/usr/bin/env bash
set -euo pipefail

IMAGE_NAME="talkwithrustgpt-win-cross:local"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
OUT_DIR="${ROOT_DIR}/artifacts/windows"

mkdir -p "${OUT_DIR}"

echo "Build image..."
docker build -f "${ROOT_DIR}/docker/win-cross/Dockerfile" -t "${IMAGE_NAME}" "${ROOT_DIR}"

echo "Run build in container..."
docker run --rm \
  -e TARGET="${TARGET:-x86_64-pc-windows-gnu}" \
  -v "${ROOT_DIR}:/src:ro" \
  -v "${OUT_DIR}:/out" \
  "${IMAGE_NAME}"
