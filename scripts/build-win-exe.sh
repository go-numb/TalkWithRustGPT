#!/usr/bin/env bash
set -euo pipefail

IMAGE_NAME="talkwithrustgpt-win-cross:local"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
OUT_DIR="${ROOT_DIR}/artifacts/windows"

mkdir -p "${OUT_DIR}"
find "${OUT_DIR}" -mindepth 1 -maxdepth 1 -type f -delete

echo "Build image..."
docker build --pull=false -f "${ROOT_DIR}/docker/win-cross/Dockerfile" -t "${IMAGE_NAME}" "${ROOT_DIR}"

echo "Run build in container..."
docker run --rm \
  --dns "${DOCKER_DNS:-100.64.100.1}" \
  -e TARGET="${TARGET:-x86_64-pc-windows-gnu}" \
  -v talkwithrustgpt-npm-cache:/root/.npm \
  -v talkwithrustgpt-cargo-registry:/opt/cargo/registry \
  -v talkwithrustgpt-cargo-git:/opt/cargo/git \
  -v talkwithrustgpt-cargo-target:/cache/target \
  -v "${ROOT_DIR}:/src:ro" \
  -v "${OUT_DIR}:/out" \
  "${IMAGE_NAME}"
