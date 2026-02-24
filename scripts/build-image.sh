#!/bin/bash
# Build the fuel bot Docker image

set -e

echo "🔨 Building fuel-bot Docker image..."
echo ""

# Build the image
podman build -t fuel-bot:latest .

if [ $? -eq 0 ]; then
  echo ""
  echo "✅ Image built successfully: fuel-bot:latest"
  echo ""
  echo "To restart the pod with the new image:"
  echo "  podman pod stop fuel-bot-pod && podman pod rm fuel-bot-pod"
  echo "  ./scripts/start-pod.sh"
  echo ""
  echo "Or use the quick restart script:"
  echo "  ./scripts/restart-pod.sh"
else
  echo ""
  echo "❌ Build failed"
  exit 1
fi
