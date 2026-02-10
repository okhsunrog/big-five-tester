set dotenv-load

server := "root@mira.local"
deploy_dir := "/opt/bigfive"

# Run dev server with hot reload
run:
    cd crates/bigfive-app && cargo leptos watch

# Build release (frontend + backend)
build:
    rm -rf target/site/pkg
    cd crates/bigfive-app && touch src/lib.rs style/tailwind.css && cargo leptos build --release

# Run cargo tests
test:
    cargo test

# Run all checks (fmt, clippy, tests)
check:
    cargo fmt --check
    cargo clippy -- -D warnings
    cargo test

# Format code
fmt:
    cargo fmt

# Lint
lint:
    cargo clippy -- -D warnings

# Deploy to server
deploy: build
    @echo "=== Deploying to {{server}} ==="
    @echo "Binary size: $(du -h target/release/bigfive-app | cut -f1)"
    @echo "Site size: $(du -sh target/site | cut -f1)"
    @test -f "target/site/pkg/bigfive-app.js" || (echo "ERROR: Frontend build failed - target/site/pkg/bigfive-app.js not found" && exit 1)
    ssh {{server}} "mkdir -p {{deploy_dir}}"
    rsync -avz --progress target/release/bigfive-app {{server}}:{{deploy_dir}}/
    rsync -avz --progress --delete target/site/ {{server}}:{{deploy_dir}}/site/
    scp bigfive.service {{server}}:/etc/systemd/system/
    ssh {{server}} "systemctl daemon-reload && systemctl enable bigfive"
    @if ! ssh {{server}} "test -f {{deploy_dir}}/.env"; then \
        echo ""; \
        echo "WARNING: {{deploy_dir}}/.env does not exist!"; \
        echo "Create it with:"; \
        echo "  ssh {{server}} 'cat > {{deploy_dir}}/.env << EOF"; \
        echo "ANTHROPIC_API_KEY=your-key-here"; \
        echo "EOF'"; \
        echo "Then run: just restart"; \
        exit 1; \
    fi
    ssh {{server}} "systemctl restart bigfive"
    @sleep 2
    ssh {{server}} "systemctl status bigfive --no-pager"
    @echo ""
    @echo "=== Deployed! ==="
    @echo "Service: https://bigfive.okhsunrog.ru"

# View server logs
logs:
    ssh {{server}} "journalctl -u bigfive -f"

# Check server status
status:
    ssh {{server}} "systemctl status bigfive --no-pager"

# Restart server
restart:
    ssh {{server}} "systemctl restart bigfive"
