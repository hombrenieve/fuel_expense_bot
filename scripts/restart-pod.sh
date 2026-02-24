#!/bin/bash
# Restart the fuel bot pod (useful after rebuilding the image)

set -e

echo "🔄 Restarting fuel bot pod..."
echo ""

# Check if pod exists
if podman pod exists fuel-bot-pod; then
  echo "Stopping existing pod..."
  podman pod stop fuel-bot-pod
  podman pod rm fuel-bot-pod
  echo "✓ Pod stopped and removed"
  echo ""
fi

# Get token from .env
TELEGRAM_TOKEN=$(grep TELEGRAM_TOKEN .env | cut -d= -f2)

if [ -z "$TELEGRAM_TOKEN" ]; then
  echo "❌ Error: TELEGRAM_TOKEN not found in .env"
  exit 1
fi

echo "Starting pod..."
export TELEGRAM_TOKEN
envsubst < pod.yaml | podman play kube -

echo ""
echo "⏳ Waiting for services to start..."
sleep 8

echo ""
echo "✅ Pod restarted successfully"
echo ""
echo "Services running:"
echo "  • Telegram Bot: fuel-bot-pod-fuel-bot-app"
echo "  • MariaDB: fuel-bot-pod-fuel-bot-db (internal only)"
echo "  • Adminer UI: http://localhost:8080"
echo ""
echo "Check logs with:"
echo "  podman logs fuel-bot-pod-fuel-bot-app"
