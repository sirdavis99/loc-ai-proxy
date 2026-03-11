# Development

## Building from Source

### Prerequisites

- Rust 1.78+ ([Install](https://rustup.rs/))
- opencode installed and configured ([Install](https://opencode.ai/))

### Build

```bash
# Clone repository
git clone https://github.com/sirdavis99/loc-ai-proxy.git
cd loc-ai-proxy

# Build debug version
cargo build

# Build release version
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

## Project Structure

```
src/
├── main.rs              # Entry point, CLI handling
├── lib.rs               # Library exports
├── config.rs            # Configuration management
├── cli.rs               # CLI commands (configure, status, etc.)
├── server.rs            # HTTP server setup
├── api/                 # OpenAI-compatible API
│   ├── mod.rs          # Route handlers
│   └── models.rs       # Request/response types
├── providers/          # Provider adapters
│   ├── mod.rs          # Provider trait and registry
│   └── opencode/       # opencode implementation
├── session/            # Session management
│   └── manager.rs      # Session lifecycle
├── models/             # Model registry
│   └── registry.rs     # Model aliases and mapping
└── utils/              # Utilities
    ├── errors.rs       # Error types
    └── logging.rs      # Logging setup
```

## Architecture

See [ARCHITECTURE.md](./ARCHITECTURE.md) for detailed system design.

## Adding a New Provider

1. Create `src/providers/{provider}/mod.rs`
2. Implement `ProviderAdapter` trait
3. Register in `server.rs`
4. Add configuration in `config.rs`
5. Update documentation

See existing `opencode` provider for reference.

## Testing

```bash
# Unit tests
cargo test

# Integration tests
cargo test --features integration

# Run with test config
LOC_AI_PROXY_CONFIG=./test-config.yaml cargo run
```

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Follow Rust naming conventions
- Document public APIs with doc comments

## Debugging

```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Enable trace logging (very verbose)
RUST_LOG=trace cargo run

# Filter specific modules
RUST_LOG=loc_ai_proxy::providers=debug cargo run
```

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create git tag: `git tag v0.x.x`
4. Push: `git push origin v0.x.x`
5. GitHub Actions will build and release

## Contributing

1. Fork the repository
2. Create feature branch: `git checkout -b feature/my-feature`
3. Commit changes: `git commit -am 'Add new feature'`
4. Push to branch: `git push origin feature/my-feature`
5. Open Pull Request

See [CONTRIBUTING.md](./CONTRIBUTING.md) for detailed guidelines.
