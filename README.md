# Big Five Personality Test

A Leptos fullstack application for taking the Big Five (IPIP-NEO-120) personality test with AI-powered analysis.

## Project Structure

```
big-five-tester/
├── crates/
│   ├── bigfive/        # Core library for Big Five test scoring
│   └── bigfive-app/    # Leptos fullstack application
```

## Development

### Prerequisites

- Rust (with `wasm32-unknown-unknown` target)
- [cargo-leptos](https://github.com/leptos-rs/cargo-leptos)
- [Tailwind CSS](https://tailwindcss.com/)

### Formatting

This project uses `rustfmt` and `leptosfmt` for code formatting.

**Important:** You need to install `leptosfmt` from git (unreleased version) to get the fix for generic component formatting:

```bash
cargo install --git https://github.com/bram209/leptosfmt.git
```

The released version (0.1.33) has a bug that breaks formatting of generic components like `<I18nRoute<Locale, _, _>>`.

Run formatters:
```bash
cargo fmt
leptosfmt crates/bigfive-app/src/**/*.rs
```

### Running

```bash
cd crates/bigfive-app
cargo leptos watch
```

### Environment Variables

- `ANTHROPIC_API_KEY` - Required for AI-powered personality analysis
