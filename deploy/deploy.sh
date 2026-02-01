#!/bin/bash
set -e

SERVER="root@mira.local"
DEPLOY_DIR="/opt/bigfive"

echo "=== Building big-five-tester for release ==="
cd "$(dirname "$0")/.."

# Clean old site files to force rebuild
rm -rf target/site/pkg

# Build release binary (from bigfive-app directory where Cargo.toml with leptos config is)
cd crates/bigfive-app
# Touch source files to force frontend rebuild
touch src/lib.rs style/tailwind.css
cargo leptos build --release
cd ../..

echo ""
echo "=== Preparing deployment package ==="

# Verify build artifacts exist
if [ ! -f "target/site/pkg/bigfive-app.js" ]; then
    echo "ERROR: Frontend build failed - target/site/pkg/bigfive-app.js not found"
    exit 1
fi

# Create temp dir with deployment files
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

# Copy binary (built in workspace target/)
cp target/release/bigfive-app "$TEMP_DIR/"

# Copy site directory (CSS, JS, WASM)
cp -r target/site "$TEMP_DIR/site"

# Show what we're deploying
echo "Frontend files:"
ls -la target/site/pkg/

echo "Binary size: $(du -h target/release/bigfive-app | cut -f1)"
echo "Site size: $(du -sh target/site | cut -f1)"

echo ""
echo "=== Syncing to $SERVER ==="

# Create directories on server
ssh "$SERVER" "mkdir -p $DEPLOY_DIR"

# Sync files
rsync -avz --progress "$TEMP_DIR/bigfive-app" "$SERVER:$DEPLOY_DIR/"
rsync -avz --progress --delete "$TEMP_DIR/site/" "$SERVER:$DEPLOY_DIR/site/"

echo ""
echo "=== Installing systemd service ==="

# Copy and enable service
scp deploy/bigfive.service "$SERVER:/etc/systemd/system/"
ssh "$SERVER" "systemctl daemon-reload && systemctl enable bigfive"

echo ""
echo "=== Checking .env file ==="

# Check if .env exists
if ! ssh "$SERVER" "test -f $DEPLOY_DIR/.env"; then
    echo ""
    echo "WARNING: $DEPLOY_DIR/.env does not exist!"
    echo "Create it with:"
    echo ""
    echo "  ssh $SERVER \"cat > $DEPLOY_DIR/.env << 'EOF'"
    echo "ANTHROPIC_API_KEY=your-key-here"
    echo "EOF\""
    echo ""
    echo "Then run: ssh $SERVER 'systemctl restart bigfive'"
    exit 1
fi

echo ""
echo "=== Restarting service ==="

ssh "$SERVER" "systemctl restart bigfive"
ssh "$SERVER" "systemctl status bigfive --no-pager"

echo ""
echo "=== Deployment complete! ==="
echo ""
echo "Service: https://bigfive.okhsunrog.ru"
echo ""
echo "Logs: ssh $SERVER 'journalctl -u bigfive -f'"
