#!/bin/bash

# EnvArchitect CLI Hot Reload Development Mode
# This watches for file changes and automatically reinstalls the global binary

echo "🔥 Starting EnvArchitect CLI in HOT RELOAD mode..."
echo "📝 Any changes to apps/cli will automatically update the global 'env-architect' command"
echo ""

cd "$(dirname "$0")"

cargo watch \
  --watch apps/cli/src \
  --watch apps/cli/Cargo.toml \
  --clear \
  --exec "install --path apps/cli --force --quiet" \
  --shell 'echo "✅ env-architect binary updated! ($(date +%H:%M:%S))"'
