#!/bin/bash
set -e

echo "Building Zenith CLI..."
cd cli && cargo build --release
cd ..

echo "Starting Zenith Data Plane..."
echo "Dashboard available at: http://localhost:8080/dashboard (Manual open file dashboard/index.html)"
echo "API available at: http://localhost:8080/status"

./cli/target/release/zenith start --config config/zenith.toml
