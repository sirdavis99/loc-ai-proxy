# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned for Phase 2
- Streaming responses (SSE)
- Ollama provider support
- OpenRouter provider support
- GitHub Copilot provider support
- Metrics and monitoring
- WebSocket support

## [0.1.0] - 2026-03-11

### Added
- Initial release
- OpenAI-compatible `/v1/chat/completions` endpoint
- opencode provider support
- Session management with automatic cleanup
- Model aliasing system
- CLI configuration commands
- Auto-detection of opencode installation
- Docker support with Dockerfile and docker-compose.yml
- Comprehensive documentation
- GitHub Actions CI/CD pipeline
- MIT License

### Features
- Non-streaming chat completions
- Session-based conversation management
- 30-minute session TTL with automatic cleanup
- Provider registry for multiple backends
- Configuration via CLI, env vars, or config file
- Health check endpoint
- Model listing endpoint
- Auto-start opencode option

## Phase 2 Roadmap

### Provider Support
- [ ] Ollama provider
- [ ] OpenRouter provider
- [ ] GitHub Copilot provider
- [ ] Google Gemini direct
- [ ] Groq direct
- [ ] Mistral direct

### Features
- [ ] Streaming responses (Server-Sent Events)
- [ ] Tool call support
- [ ] Multi-modal support (images)
- [ ] Request queueing
- [ ] Load balancing across providers

### Operations
- [ ] Prometheus metrics
- [ ] Admin dashboard
- [ ] API key management
- [ ] Usage analytics

[unreleased]: https://github.com/sirdavis99/loc-ai-proxy/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/sirdavis99/loc-ai-proxy/releases/tag/v0.1.0
