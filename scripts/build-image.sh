#!/bin/bash
# Build the fuel bot Docker image

set -e

CLEAN_BUILD=false

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --clean)
      CLEAN_BUILD=true
      shift
      ;;
    *)
      echo "Unknown option: $1"
      echo "Usage: $0 [--clean]"
      echo ""
      echo "Options:"
      echo "  --clean    Force a clean rebuild without cache"
      exit 1
      ;;
  esac
done

if [ "$CLEAN_BUILD" = true ]; then
  echo "🧹 Clean build mode enabled"
  echo ""
  
  # Clean local artifacts
  if [ -d "target" ]; then
    echo "Removing local target/ directory..."
    rm -rf target
    echo "✓ Local build cache cleared"
    echo ""
  fi
  
  echo "🔨 Building image with --no-cache..."
  podman build --no-cache -t fuel-bot:latest .
else
  echo "🔨 Building fuel-bot Docker image..."
  echo ""
  podman build -t fuel-bot:latest .
fi

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
