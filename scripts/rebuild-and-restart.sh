#!/bin/bash
# Build the image and restart the pod in one command

set -e

echo "🚀 Building and restarting fuel bot..."
echo ""

# Build the image
./scripts/build-image.sh

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Restart the pod
./scripts/restart-pod.sh
