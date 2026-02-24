#!/bin/bash
# Start the fuel bot pod with environment variable substitution

set -e

# Check if TELEGRAM_TOKEN is set
if [ -z "$TELEGRAM_TOKEN" ]; then
  echo "❌ Error: TELEGRAM_TOKEN environment variable is not set"
  echo ""
  echo "Set it with:"
  echo "  export TELEGRAM_TOKEN='your_bot_token_here'"
  echo ""
  echo "Or load from .env:"
  echo "  source .env"
  echo "  export TELEGRAM_TOKEN"
  exit 1
fi

echo "Starting fuel bot pod..."
echo "Using TELEGRAM_TOKEN: ${TELEGRAM_TOKEN:0:10}..."

# Substitute environment variables and start the pod (includes ConfigMap)
envsubst < pod-with-configmap.yaml | podman play kube -

echo ""
echo "✓ Pod started successfully"
echo ""
echo "Services running:"
echo "  • Telegram Bot: fuel-bot-pod-fuel-bot-app"
echo "  • MariaDB: fuel-bot-pod-fuel-bot-db (internal only)"
echo "  • Adminer UI: http://localhost:8080"
echo ""
echo "Check status with:"
echo "  podman pod ps"
echo "  podman ps --filter pod=fuel-bot-pod"
echo ""
echo "View logs with:"
echo "  podman logs fuel-bot-pod-fuel-bot-app"
