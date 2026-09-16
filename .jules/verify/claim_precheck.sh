#!/usr/bin/env bash
# Enforces pre-claiming tasks and manages TTL expiration for claims.
set -euo pipefail

CLAIMS_FILE=".jules/claims.json"
mkdir -p .jules/verify .jules/context

if [ ! -f "$CLAIMS_FILE" ]; then
  echo '{"claims":[]}' > "$CLAIMS_FILE"
fi

PRUNE_EXPIRED=false
CRATE=""
TASK=""
SESSION_HASH="${SESSION_HASH:-$(date -u +%Y%m%d%H%M%S | sha256sum | head -c 8)}"

while [[ $# -gt 0 ]]; do
  case $1 in
    --prune-expired)
      PRUNE_EXPIRED=true
      shift
      ;;
    --crate)
      CRATE="$2"
      shift 2
      ;;
    --task)
      TASK="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done

NOW_EPOCH=$(date +%s)
TTL_SECONDS=7200 # 2 hours TTL

if [ "$PRUNE_EXPIRED" = true ]; then
  jq --argjson now "$NOW_EPOCH" '
    if (type == "object" and .claims) then
      .claims |= map(
        if (.active == true and (((.created_at // 0) + (.ttl_seconds // 7200)) < $now)) then
          .active = false | .released_at = "TTL-EXPIRED"
        else
          .
        end
      )
    else
      .
    end' "$CLAIMS_FILE" > "${CLAIMS_FILE}.tmp"
  mv "${CLAIMS_FILE}.tmp" "$CLAIMS_FILE"
  echo "Expired claims pruned."
  if [ -z "$CRATE" ]; then
    exit 0
  fi
fi

if [ -n "$CRATE" ]; then
  EXISTING=$(jq -r --arg crate "$CRATE" --arg session "$SESSION_HASH" '
    (if (type == "object" and .claims) then .claims else . end)
    | .[] | select((.krate == $crate or .crate == $crate) and (.active == true) and (.session_id != $session and .session != $session)) | (.session_id // .session)' "$CLAIMS_FILE")

  if [ -n "$EXISTING" ]; then
    echo "❌ CLAIM CONFLICT: Crate '\''$CRATE'\'' is already claimed by active session '\''$EXISTING'\''"
    exit 1
  fi

  TASK_DESC="${TASK:-work_on_$CRATE}"
  TS_ISO=$(date -u +%Y-%m-%dT%H:%M:%SZ)

  UPDATED_CLAIMS=$(jq --arg crate "$CRATE" --arg session "$SESSION_HASH" --arg task "$TASK_DESC" --arg ts "$TS_ISO" --argjson now "$NOW_EPOCH" --argjson ttl "$TTL_SECONDS" '
    if (type == "object" and .claims) then
      .claims += [{krate: $crate, crate: $crate, issue: $task, timestamp: $ts, session_id: $session, session: $session, active: true, created_at: $now, ttl_seconds: $ttl}]
    else
      . + [{krate: $crate, crate: $crate, issue: $task, timestamp: $ts, session_id: $session, session: $session, active: true, created_at: $now, ttl_seconds: $ttl}]
    end' "$CLAIMS_FILE")
  echo "$UPDATED_CLAIMS" > "$CLAIMS_FILE"
  echo "✅ Claim registered for crate '\''$CRATE'\'' by session '\''$SESSION_HASH'\''"
fi
