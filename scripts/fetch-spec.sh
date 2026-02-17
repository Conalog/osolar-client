#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SPEC_URL="https://registry.scalar.com/@bworx/apis/osolar-link-open-api"
OUT_PATH="${ROOT_DIR}/specs/osolar-link-openapi.json"

curl -sSL "${SPEC_URL}" -o "${OUT_PATH}"
echo "Saved spec to ${OUT_PATH}"
