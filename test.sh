#!/usr/bin/env bash
set -euo pipefail

GREPTIMEDB_URL="http://localhost:4000/v1/sql"
MAX_RETRIES=30
RETRY_INTERVAL=5

echo "Waiting for data in GreptimeDB..."

for i in $(seq 1 "$MAX_RETRIES"); do
    response=$(curl -s -w "\n%{http_code}" "$GREPTIMEDB_URL" \
        --data-urlencode "sql=SELECT * FROM motor_driver ORDER BY ts DESC LIMIT 5" 2>/dev/null || true)

    http_code=$(echo "$response" | tail -1)
    body=$(echo "$response" | sed '$d')

    if [ "$http_code" = "200" ] && echo "$body" | grep -q "mc01"; then
        echo "SUCCESS: Found data in motor_driver table"
        echo "$body" | python3 -m json.tool 2>/dev/null || echo "$body"
        exit 0
    fi

    echo "Attempt $i/$MAX_RETRIES: no data yet (HTTP $http_code), retrying in ${RETRY_INTERVAL}s..."
    sleep "$RETRY_INTERVAL"
done

echo "FAILED: No data found in motor_driver table after $((MAX_RETRIES * RETRY_INTERVAL))s"
exit 1
