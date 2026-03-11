# Contributing to loc-ai-proxy

Thank you for your interest in contributing! This document provides guidelines and information for contributors.

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow

## How to Contribute

### Reporting Bugs

1. Check if the bug has already been reported in [Issues](https://github.com/sirdavis99/loc-ai-proxy/issues)
2. If not, create a new issue using the bug report template
3. Include:
   - Clear description of the bug
   - Steps to reproduce
   - Expected vs actual behavior
   - Environment details (OS, versions)
   - Relevant logs

### Suggesting Features

1. Check existing issues and discussions
2. Open a new issue with the feature request template
3. Explain the use case and why it would be valuable

### Pull Requests

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run tests: `cargo test`
5. Format code: `cargo fmt`
6. Run clippy: `cargo clippy`
7. Commit with clear messages
8. Push to your fork
9. Open a Pull Request

#### PR Requirements

- [ ] Code passes `cargo test`
- [ ] Code is formatted with `cargo fmt`
- [ ] No clippy warnings
- [ ] Documentation updated if needed
- [ ] CHANGELOG.md updated
- [ ] Commit messages are clear

## Development Setup

See [DEVELOPMENT.md](./DEVELOPMENT.md) for detailed setup instructions.

Quick start:
```bash
git clone https://github.com/sirdavis99/loc-ai-proxy.git
cd loc-ai-proxy
cargo build
cargo test
```

## Code Style

### Rust Conventions

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Document public APIs with doc comments (`///`)

### Naming

- Functions/variables: `snake_case`
- Types/traits: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`

### Error Handling

- Use `thiserror` for custom errors
- Use `anyhow` for application errors
- Propagate errors with `?`
- Provide context with `.context()`

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feature() {
        assert_eq!(actual, expected);
    }
}
```

### Integration Tests

Place in `tests/` directory:

```rust
// tests/integration_test.rs
use loc_ai_proxy::*;

#[tokio::test]
async fn test_endpoint() {
    // Test code
}
```

Run with:
```bash
cargo test
cargo test --features integration
```

## Documentation

- Update README.md for user-facing changes
- Update ARCHITECTURE.md for design changes
- Add examples for new features
- Keep CHANGELOG.md updated

## Commit Messages

Format: `<type>: <description>`

Types:
- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation
- `style:` Formatting
- `refactor:` Code restructuring
- `test:` Tests
- `chore:` Maintenance

Examples:
```
feat: add streaming support
fix: resolve session timeout issue
docs: update Pi setup guide
```

## Adding a Provider

See [DEVELOPMENT.md](./DEVELOPMENT.md) for the provider implementation guide.

## Questions?

- Open a [Discussion](https://github.com/sirdavis99/loc-ai-proxy/discussions)
- Join [Discord](https://discord.gg/nKXTsAcmbT)

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

Thank you for contributing! 🎉
