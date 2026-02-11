# Big Five Tester

A Rust + Leptos fullstack web app for taking the Big Five personality test (IPIP-NEO-120), calculating your scores (domains + facets), and optionally generating an AI-written narrative analysis.

## Features

- **IPIP-NEO-120** scoring (5 domains, 30 facets) via `crates/bigfive`
- **Leptos fullstack app** (`crates/bigfive-app`) with an Axum SSR backend
- **EN/RU** localization with progress persisted in browser `localStorage`
- **AI analysis pipeline** with:
  - multiple model presets from `ai_config.toml` (user-selectable in UI)
  - providers: Anthropic API and OpenAI-compatible APIs (OpenRouter/OpenAI/Ollama/etc.)
  - adaptive thinking support (configurable effort level)
  - optional prompt-injection safeguard step
  - optional translate step when model output language != UI language
- **Shareable results**: save a snapshot of your scores + AI analysis to an embedded SQLite database (turso) and share via link. Each share creates an immutable snapshot with a unique URL.
- **Export as PDF** via browser print dialog
- **Rate limiting** with IP-based tracking (configurable per-endpoint)

## Project Structure

```
big-five-tester/
├── crates/
│   ├── bigfive/        # Core library for Big Five test scoring
│   └── bigfive-app/    # Leptos fullstack application
├── ai_config.toml      # AI model presets (see ai_config.example.toml)
└── justfile            # Common dev/build/deploy commands
```

## Quickstart (Development)

### Prerequisites

- Rust **nightly** (see `rust-toolchain.toml`) with the `wasm32-unknown-unknown` target
- `cargo-leptos` (for `cargo leptos watch/build`)
- `just` (optional, for the shortcuts in `justfile`)
- No JS toolchain required: Tailwind is driven by `cargo-leptos` via the `tailwind-input-file` site parameter.

### Setup

1) Configure API keys and AI presets:

```bash
cp .env.example .env
cp ai_config.example.toml ai_config.toml
```

2) Run the dev server (hot reload):

```bash
just run
```

Then open `http://127.0.0.1:3032`.

## Configuration

### `ai_config.toml`

The app loads AI configuration from:

- `AI_CONFIG_PATH` (if set), otherwise `./ai_config.toml`

See `ai_config.example.toml` for a fully documented configuration file. You can define multiple `[[models]]` presets (these show up in the UI) and an optional `[safeguard]` model used for prompt-injection detection on user-provided context.

### Environment variables

Keys are read based on `api_key_env` in `ai_config.toml`. The included `.env.example` uses:

- `OPENROUTER_API_KEY` (default for OpenAI-compatible presets in `ai_config.example.toml`)
- `ANTHROPIC_API_KEY` (if you use Anthropic presets)
- `AI_CONFIG_PATH` (optional; defaults to `./ai_config.toml`)
- `DATABASE_PATH` (optional; defaults to `data/bigfive.db`)
- `RUST_LOG` (optional; e.g. `info`, `debug`)

## Common commands

This repo uses `just` (see `justfile`):

```bash
just run     # dev server (cargo leptos watch)
just check   # fmt + clippy -D warnings + tests
just build   # release build (frontend + backend)
just deploy  # build + deploy to remote server via rsync
```

## Formatting

This project uses `rustfmt` and `leptosfmt`.

**Important:** install `leptosfmt` from git (unreleased) to get the fix for generic component formatting:

```bash
cargo install --git https://github.com/bram209/leptosfmt.git
```

Run formatters:
```bash
cargo fmt
leptosfmt crates/bigfive-app/src/**/*.rs
```

## Deployment (systemd)

There's a simple deployment workflow baked into `justfile`:

- `just deploy` builds a release and syncs `target/release/bigfive-app` + `target/site/` to a remote host
- `bigfive.service` is copied to `/etc/systemd/system/` and the service is restarted

The defaults assume:

- remote host: `root@mira.local`
- deploy dir: `/opt/bigfive`

Adjust `server` / `deploy_dir` at the top of `justfile` to match your environment.

## License

MIT
