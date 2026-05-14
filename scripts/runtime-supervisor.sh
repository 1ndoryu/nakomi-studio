#!/bin/sh
# [145A-1] External HTTP supervisor for Rust containers.
# It runs outside Tokio, so it still reacts if the Rust runtime accepts TCP but stops polling HTTP.

set -eu

PORT="${PORT:-3000}"
HEALTH_PATH="${GLORY_SUPERVISOR_HEALTH_PATH:-/healthz}"
INTERVAL="${GLORY_SUPERVISOR_INTERVAL:-30}"
TIMEOUT="${GLORY_SUPERVISOR_TIMEOUT:-4}"
RETRIES="${GLORY_SUPERVISOR_RETRIES:-3}"
START_PERIOD="${GLORY_SUPERVISOR_START_PERIOD:-90}"
URL="http://127.0.0.1:${PORT}${HEALTH_PATH}"

log() {
    printf '%s [runtime-supervisor] %s\n' "$(date -u +"%Y-%m-%dT%H:%M:%SZ")" "$*"
}

dump_state() {
    log "process status before restart"
    cat "/proc/${APP_PID}/status" 2>/dev/null || true
    log "tcp snapshot before restart"
    cat /proc/net/tcp /proc/net/tcp6 2>/dev/null | head -120 || true
    log "fd count before restart: $(ls "/proc/${APP_PID}/fd" 2>/dev/null | wc -l || true)"
}

shutdown() {
    log "received stop signal, forwarding to app pid ${APP_PID}"
    kill -TERM "${APP_PID}" 2>/dev/null || true
    wait "${APP_PID}" 2>/dev/null || true
    exit 0
}

/app/glory-backend &
APP_PID="$!"
trap shutdown INT TERM
log "started app pid ${APP_PID}; probing ${URL} after ${START_PERIOD}s"

elapsed=0
while [ "${elapsed}" -lt "${START_PERIOD}" ]; do
    if ! kill -0 "${APP_PID}" 2>/dev/null; then
        wait "${APP_PID}"
        exit "$?"
    fi
    sleep 1
    elapsed=$((elapsed + 1))
done

failures=0
while kill -0 "${APP_PID}" 2>/dev/null; do
    if curl -fsS --max-time "${TIMEOUT}" "${URL}" >/dev/null; then
        if [ "${failures}" -gt 0 ]; then
            log "health recovered after ${failures} failed probe(s)"
        fi
        failures=0
    else
        failures=$((failures + 1))
        log "health probe failed (${failures}/${RETRIES}) url=${URL}"
        if [ "${failures}" -ge "${RETRIES}" ]; then
            dump_state
            log "terminating frozen app so Docker restart policy can recover it"
            kill -TERM "${APP_PID}" 2>/dev/null || true
            sleep 5
            kill -KILL "${APP_PID}" 2>/dev/null || true
            wait "${APP_PID}" 2>/dev/null || true
            exit 1
        fi
    fi
    sleep "${INTERVAL}"
done

wait "${APP_PID}"
exit "$?"
