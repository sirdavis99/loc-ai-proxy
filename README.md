# loc-ai-proxy

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.78%2B-orange.svg)](https://www.rust-lang.org)
[![Docker](https://img.shields.io/badge/docker-ready-blue.svg)](https://www.docker.com)

**A unified OpenAI-compatible proxy for local AI agents** — enabling tools like Pi to access multiple AI providers through a single interface.

## 🎯 Purpose

`loc-ai-proxy` acts as a bridge between OpenAI-compatible clients (like [Pi](https://pi.dev/)) and various AI inference engines. Currently supports opencode, with plans to add more providers.

### The Problem

- **Pi** and other OpenAI-compatible tools can't directly use opencode's providers
- **opencode** has access to 11+ providers (Google, Groq, Mistral, NVIDIA, OpenRouter, etc.) but doesn't expose OpenAI-compatible endpoints
- Managing API keys for 10+ providers is cumbersome

### The Solution

`loc-ai-proxy` provides:
- ✅ **OpenAI-compatible API** (`/v1/chat/completions`)
- ✅ **Unified interface** for all providers
- ✅ **Session management** (conversations maintain context)
- ✅ **Auto-detection** of installed tools
- ✅ **Docker support** for easy deployment

## 🚀 Quick Start

### Installation

```bash
# Via cargo
cargo install loc-ai-proxy

# Or clone and build
git clone https://github.com/sirdavis99/loc-ai-proxy.git
cd loc-ai-proxy
cargo build --release
```

### Configuration

```bash
# Configure opencode integration
locaiproxy configure opencode

# Or manual configuration
export OPENCODE_URL=http://127.0.0.1:4096
export LOC_AI_PROXY_PORT=9110
```

### Running

```bash
# Start the proxy
locaiproxy

# Or with Docker
docker run -p 9110:9110 sirdavis99/loc-ai-proxy:latest
```

### Using with Pi

Add to your Pi `~/.pi/agent/models.json`:

```json
{
  "providers": {
    "loc-ai-proxy": {
      "baseUrl": "http://localhost:9110/v1",
      "api": "openai-completions",
      "models": [
        { "id": "opencode/anthropic/claude-3.5-sonnet", "name": "Claude 3.5 Sonnet" },
        { "id": "opencode/google/gemini-2.5-pro", "name": "Gemini 2.5 Pro" },
        { "id": "opencode/groq/llama-3.3-70b", "name": "Llama 3.3 70B" }
      ]
    }
  }
}
```

## 📋 Supported Providers

### Current (Phase 1)
- [x] **opencode** - All 300+ models via opencode's gateway

### Planned (Phase 2+)
- [ ] **ollama** - Local models
- [ ] **openrouter** - Aggregated API
- [ ] **github-copilot** - Copilot models
- [ ] **google** - Gemini API direct
- [ ] **groq** - Fast inference
- [ ] **mistral** - Mistral models
- [ ] **nvidia** - NIM endpoints
- [ ] **fireworks** - Fireworks AI
- [ ] **inception** - Mercury models

## 🏗️ Architecture

```
Pi Terminal                    loc-ai-proxy                     Provider
┌──────────┐                 ┌──────────────┐                ┌──────────┐
│ Request  │ ──OpenAI API──▶│ API Layer    │                │          │
│          │                 │  (axum)      │                │          │
└──────────┘                 └──────┬───────┘                │          │
     ▲                              │                        │          │
     │                              ▼                        │          │
     │                       ┌──────────────┐                  │          │
     │                       │ Session      │                  │          │
     │                       │ Manager      │                  │          │
     │                       └──────┬───────┘                  │          │
     │                              │                        │          │
     │                              ▼                        │          │
     │                       ┌──────────────┐    Provider     │          │
     └──────────────────────│ Provider     │◄───────────────│ opencode │
                             │ Adapter      │   HTTP/CLI     │          │
                             └──────────────┘                └──────────┘
```

**Key Components:**
- **API Layer**: OpenAI-compatible REST endpoints
- **Session Manager**: Maintains conversation context per client
- **Provider Adapters**: Pluggable adapters for different backends
- **Model Registry**: Dynamic model discovery and mapping

## 📚 Documentation

- [Architecture](./docs/ARCHITECTURE.md) - System design and data flow
- [Provider Setup](./docs/PROVIDER_SETUP.md) - Configure each provider
- [Pi Integration](./docs/PI_SETUP.md) - Using with Pi coding agent
- [API Reference](./docs/API.md) - OpenAI-compatible endpoints
- [Development](./docs/DEVELOPMENT.md) - Contributing guidelines

## 🛠️ Development

### Prerequisites

- Rust 1.78+
- opencode installed (for opencode provider)

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run with logging
RUST_LOG=debug cargo run
```

### Testing

```bash
# Unit tests
cargo test

# Integration tests
cargo test --features integration
```

### Docker

```bash
# Build image
docker build -t loc-ai-proxy .

# Run container
docker run -p 9110:9110 -e OPENCODE_URL=http://host.docker.internal:4096 loc-ai-proxy
```

## 🤝 Contributing

We welcome contributions! See our [Contributing Guide](./docs/CONTRIBUTING.md) for details.

### Phase 2 Issues

Check out [GitHub Issues](https://github.com/sirdavis99/loc-ai-proxy/issues) labeled `phase-2` for upcoming features.

## 📄 License

MIT License - see [LICENSE](./LICENSE) file.

## 🙏 Acknowledgments

- [Pi](https://pi.dev/) - The minimal coding agent that inspired this project
- [opencode](https://opencode.ai/) - Providing access to 300+ models
- Rust community - For excellent async ecosystem

## 📞 Support

- [GitHub Issues](https://github.com/sirdavis99/loc-ai-proxy/issues)
- [Discussions](https://github.com/sirdavis99/loc-ai-proxy/discussions)

---

**Made with ❤️ for the local AI community**
