#!/usr/bin/env bash
# Deploy the board to Cloudflare Workers (workers.dev). Needs a token with Workers Scripts:Edit
# (sent as Authorization: Bearer $CF_API_TOKEN, or injected by a proxy). Optional: CF_ACCOUNT_ID, WORKER_NAME.
set -euo pipefail
cd "$(dirname "$0")/.."
API=https://api.cloudflare.com/client/v4
AUTH=(-H "Authorization: Bearer ${CF_API_TOKEN:-injected-by-proxy}")
NAME=${WORKER_NAME:-surety}
node scripts/build-worker.js
ACC=${CF_ACCOUNT_ID:-$(curl -s "${AUTH[@]}" $API/accounts | node -pe 'JSON.parse(require("fs").readFileSync(0)).result[0].id')}
curl -s "${AUTH[@]}" -X PUT "$API/accounts/$ACC/workers/scripts/$NAME" \
  -F 'metadata={"main_module":"worker.js","compatibility_date":"2026-10-01"};type=application/json' \
  -F "worker.js=@web/dist/worker.js;type=application/javascript+module" | node -pe 'const j=JSON.parse(require("fs").readFileSync(0));j.success?"uploaded":JSON.stringify(j.errors)'
curl -s "${AUTH[@]}" -X POST "$API/accounts/$ACC/workers/scripts/$NAME/subdomain" -H 'content-type: application/json' -d '{"enabled":true}' >/dev/null
SUB=$(curl -s "${AUTH[@]}" "$API/accounts/$ACC/workers/subdomain" | node -pe 'JSON.parse(require("fs").readFileSync(0)).result.subdomain')
echo "https://$NAME.$SUB.workers.dev/"
