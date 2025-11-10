#!/bin/bash
# Script to copy marker tool to app bundle during Xcode build
# This script should be run as a Build Phase in Xcode

set -e

# Get the app bundle path from Xcode environment variables
APP_BUNDLE="${BUILT_PRODUCTS_DIR}/${PRODUCT_NAME}.app"
MARKER_DEST_DIR="${APP_BUNDLE}/Contents/Resources/marker"

# Create marker directory if it doesn't exist
mkdir -p "${MARKER_DEST_DIR}"

# Try to find marker tool from various locations
MARKER_SOURCE=""

# 1. Check if MARKER_TOOL_PATH environment variable is set
if [ -n "${MARKER_TOOL_PATH}" ] && [ -f "${MARKER_TOOL_PATH}" ]; then
    MARKER_SOURCE="${MARKER_TOOL_PATH}"
    echo "Using marker tool from MARKER_TOOL_PATH: ${MARKER_SOURCE}"
# 2. Check project resources directory
elif [ -f "${PROJECT_DIR}/../../resources/marker/marker" ]; then
    MARKER_SOURCE="${PROJECT_DIR}/../../resources/marker/marker"
    echo "Using marker tool from resources/marker/marker"
# 3. Check tools directory
elif [ -f "${PROJECT_DIR}/../../tools/marker/marker" ]; then
    MARKER_SOURCE="${PROJECT_DIR}/../../tools/marker/marker"
    echo "Using marker tool from tools/marker/marker"
# 4. Check system PATH
elif command -v marker >/dev/null 2>&1; then
    MARKER_SOURCE=$(command -v marker)
    echo "Using marker tool from system PATH: ${MARKER_SOURCE}"
fi

# Copy marker tool if found
if [ -n "${MARKER_SOURCE}" ] && [ -f "${MARKER_SOURCE}" ]; then
    cp "${MARKER_SOURCE}" "${MARKER_DEST_DIR}/marker"
    chmod +x "${MARKER_DEST_DIR}/marker"
    echo "✓ Marker tool copied to: ${MARKER_DEST_DIR}/marker"
else
    echo "⚠ Warning: Marker tool not found. Expected locations:"
    echo "  - MARKER_TOOL_PATH environment variable"
    echo "  - ${PROJECT_DIR}/../../resources/marker/marker"
    echo "  - ${PROJECT_DIR}/../../tools/marker/marker"
    echo "  - System PATH"
    echo "Marker tool will not be included in the app bundle."
    exit 0  # Don't fail the build if marker is not found
fi

