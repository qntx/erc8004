#!/usr/bin/env bash
# Cross-language anvil end-to-end: deploy the ERC-8004 registries, then run the
# TypeScript and Rust suites against the same chain.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PORT=8545
RPC="http://127.0.0.1:${PORT}"
DEPLOYMENT_JSON="${ROOT}/target/e2e/deployment.json"
# anvil default account #0 (deployer/owner)
DEPLOYER_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"

mkdir -p "${ROOT}/target/e2e"

ANVIL_PID=""
cleanup() {
  if [ -n "${ANVIL_PID}" ]; then
    kill "${ANVIL_PID}" 2>/dev/null || true
    wait "${ANVIL_PID}" 2>/dev/null || true
  fi
}
trap cleanup EXIT

# a previous run may have left the port busy — refuse instead of stealing it
if nc -z 127.0.0.1 "${PORT}" 2>/dev/null; then
  echo "e2e: port ${PORT} already in use — stop the existing anvil first" >&2
  exit 1
fi

anvil --port "${PORT}" --chain-id 31337 --silent &
ANVIL_PID=$!

# wait for the node
for _ in $(seq 1 50); do
  if cast chain-id --rpc-url "${RPC}" >/dev/null 2>&1; then break; fi
  sleep 0.1
done
cast chain-id --rpc-url "${RPC}" >/dev/null 2>&1 || { echo "e2e: anvil did not start" >&2; exit 1; }

echo "e2e: installing soldeer dependencies"
(cd "${ROOT}/e2e" && forge soldeer install)

echo "e2e: deploying registries"
(cd "${ROOT}/e2e" && forge script script/Deploy.s.sol:Deploy \
  --rpc-url "${RPC}" --broadcast --private-key "${DEPLOYER_KEY}" --silent)

[ -f "${DEPLOYMENT_JSON}" ] || { echo "e2e: deployment.json missing" >&2; exit 1; }
echo "e2e: deployment -> ${DEPLOYMENT_JSON}"

echo "e2e: typescript suite"
ERC8004_E2E_DEPLOYMENT="${DEPLOYMENT_JSON}" ERC8004_E2E_RPC="${RPC}" \
  bun run --cwd "${ROOT}/packages/erc8004" test:e2e

echo "e2e: rust suite"
ERC8004_E2E_DEPLOYMENT="${DEPLOYMENT_JSON}" ERC8004_E2E_RPC="${RPC}" \
  cargo test -p erc8004 --all-features --test e2e -- --ignored --nocapture --test-threads=1

echo "e2e: done"
