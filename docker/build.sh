#!/bin/bash
echo "🚀 Building Solana program..."

# Start services
docker-compose up -d solana-validator

# Wait for validator to be ready
echo "⏳ Waiting for Solana validator to be ready..."
sleep 10

# Build the program
echo "🔨 Building program..."
docker-compose run --rm builder bash -c "
  export PATH=/root/.local/share/solana/install/active_release/bin:\$PATH &&
  cargo build-bpf
"

echo "✅ Build complete! Check target/deploy/ directory"