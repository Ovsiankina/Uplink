#!/bin/bash

# Script to run Uplink with two different users simultaneously
# User 1 uses ~/.uplink (default)
# User 2 uses ~/.uplink_user2 (custom)

set -e

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
USER2_PATH="$HOME/.uplink_user2"

echo "Starting Uplink with two users..."
echo "User 1: Using default path (~/.uplink)"
echo "User 2: Using path ($USER2_PATH)"
echo ""

# Create temp file for pids
PIDFILE=$(mktemp)
trap "rm -f $PIDFILE" EXIT

# Start User 1 in the background
echo "[User 1] Starting in background..."
cd "$PROJECT_DIR"
cargo run --bin uplink &
USER1_PID=$!
echo "$USER1_PID" >> "$PIDFILE"

# Give User 1 time to start
sleep 3

# Start User 2 in the background
echo "[User 2] Starting in background..."
cd "$PROJECT_DIR"
cargo run --bin uplink -- --path "$USER2_PATH" &
USER2_PID=$!
echo "$USER2_PID" >> "$PIDFILE"

echo ""
echo "✓ Both instances started!"
echo ""
echo "User 1 PID: $USER1_PID"
echo "User 2 PID: $USER2_PID"
echo ""
echo "To stop both instances, press Ctrl+C or run:"
echo "  kill $USER1_PID $USER2_PID"
echo ""

# Wait for both processes
wait $USER1_PID $USER2_PID 2>/dev/null || true
