# Provider Setup Guide

This guide explains how to configure each provider with loc-ai-proxy.

## Table of Contents

- [opencode](#opencode) (Current)
- [Future Providers](#future-providers)
  - [Ollama](#ollama) (Phase 2)
  - [OpenRouter](#openrouter) (Phase 2)
  - [GitHub Copilot](#github-copilot) (Phase 2)

---

## opencode

**Status:** ✅ Supported (Phase 1)

opencode provides access to 300+ models from 11+ providers through a single interface.

### Prerequisites

1. opencode installed: [Installation Guide](https://opencode.ai/docs/installation)
2. opencode authenticated (providers configured)

### Configuration

#### Auto-Configuration (Recommended)

```bash
# Interactive setup
locaiproxy configure opencode

# Or with explicit path
locaiproxy configure opencode --path /usr/local/bin/opencode
```

This will:
1. Detect opencode installation
2. Test connection to opencode server
3. Optionally auto-start opencode if not running
4. Save configuration

#### Manual Configuration

**Option 1: Environment Variables**

```bash
export OPENCODE_URL=http://127.0.0.1:4096
export OPENCODE_AUTO_START=true
export OPENCODE_TIMEOUT=120
```

**Option 2: Config File**

Create `~/.config/loc-ai-proxy/config.yaml`:

```yaml
providers:
  opencode:
    enabled: true
    url: http://127.0.0.1:4096
    auto_start: true
    timeout_seconds: 120
    health_check_interval: 30
```

### Connection Methods

loc-ai-proxy tries these methods in order:

1. **HTTP API** (preferred)
   - Connects to running opencode server
   - Full session management support
   - Fastest response times

2. **CLI Fallback** (if HTTP unavailable)
   - Uses `opencode -p` command
   - Stateless (no session management)
   - Slower but works without server

### Verification

```bash
# Check if opencode is detected
locaiproxy status

# Should show:
# opencode: ✓ Connected (http://127.0.0.1:4096)
# Available models: 300+

# Test a specific model
curl -X POST http://localhost:9110/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "opencode/anthropic/claude-3.5-sonnet",
    "messages": [{"role": "user", "content": "Hello"}]
  }'
```

### Available Models

opencode supports 11 provider categories:

```bash
# List all available models via opencode
opencode models

# Or query via proxy
curl http://localhost:9110/v1/models | jq '.data[].id'
```

**Popular Models:**

| Model ID | Provider | Best For |
|----------|----------|----------|
| `opencode/anthropic/claude-3.5-sonnet` | Anthropic | General coding, reasoning |
| `opencode/google/gemini-2.5-pro` | Google | Long context, analysis |
| `opencode/groq/llama-3.3-70b` | Groq | Speed, local inference |
| `opencode/mistral/mistral-large` | Mistral | European languages |
| `opencode/openrouter/deepseek/deepseek-chat` | DeepSeek | Cost-effective |
| `opencode/nvidia/meta/llama-3.3-70b` | NVIDIA | GPU-accelerated |

### Troubleshooting

#### "opencode not found"

**Cause:** opencode not in PATH or not installed

**Solution:**
```bash
# Check if installed
which opencode

# If not found, install:
# macOS:
brew install opencode-ai/tap/opencode

# Linux:
curl -fsSL https://opencode.ai/install | bash

# Or specify full path
locaiproxy configure opencode --path /path/to/opencode
```

#### "Cannot connect to opencode"

**Cause:** opencode server not running

**Solution:**
```bash
# Start opencode server
opencode serve

# Or enable auto-start
export OPENCODE_AUTO_START=true
locaiproxy
```

#### "Authentication failed"

**Cause:** opencode not authenticated with providers

**Solution:**
```bash
# Authenticate opencode
opencode auth

# Or configure providers in opencode
opencode providers add anthropic
opencode providers add google
# etc.
```

### Advanced Configuration

#### Session Management

```yaml
providers:
  opencode:
    session:
      ttl_minutes: 30          # Session expiration
      cleanup_interval: 300    # Cleanup run every 5 minutes
      max_concurrent: 100      # Max sessions per instance
```

#### Connection Pooling

```yaml
providers:
  opencode:
    connection:
      pool_size: 10           # HTTP connection pool
      timeout_seconds: 120    # Request timeout
      retry_attempts: 3       # Retries on failure
      retry_delay_ms: 1000    # Delay between retries
```

#### Health Checks

```yaml
providers:
  opencode:
    health_check:
      enabled: true
      interval_seconds: 30
      timeout_seconds: 5
      unhealthy_threshold: 3
      healthy_threshold: 2
```

### Docker Configuration

When running loc-ai-proxy in Docker with opencode on host:

```yaml
# docker-compose.yml
version: '3.8'
services:
  proxy:
    image: sirdavis99/loc-ai-proxy:latest
    ports:
      - "9110:9110"
    environment:
      - OPENCODE_URL=http://host.docker.internal:4096
      - RUST_LOG=info
    extra_hosts:
      - "host.docker.internal:host-gateway"
```

Note: `host.docker.internal` requires Docker Desktop. For Linux Docker:
```bash
# Use host network mode
OPENCODE_URL=http://172.17.0.1:4096  # Docker bridge IP
```

---

## Future Providers

These providers are planned for Phase 2 and beyond.

### Ollama

**Status:** 🚧 Planned (Phase 2)

Direct connection to local Ollama instance for local model inference.

**Planned Configuration:**

```yaml
providers:
  ollama:
    enabled: true
    url: http://127.0.0.1:11434
    models:
      - llama3.3:70b
      - deepseek-v3
      - qwen2.5:14b
```

### OpenRouter

**Status:** 🚧 Planned (Phase 2)

Direct connection to OpenRouter API for 150+ models.

**Planned Configuration:**

```yaml
providers:
  openrouter:
    enabled: true
    api_key: ${OPENROUTER_API_KEY}
    base_url: https://openrouter.ai/api/v1
```

### GitHub Copilot

**Status:** 🚧 Planned (Phase 2)

Use GitHub Copilot models through loc-ai-proxy.

**Planned Configuration:**

```yaml
providers:
  github-copilot:
    enabled: true
    # Uses GitHub token from env: GITHUB_TOKEN
    # or ~/.config/github-copilot/hosts.json
```

---

## Provider Comparison

| Provider | Speed | Cost | Setup | Best For |
|----------|-------|------|-------|----------|
| opencode | Medium | API keys needed* | Easy | Unified access to all |
| Ollama | Fast | Free (local) | Medium | Privacy, offline use |
| OpenRouter | Medium | Pay per use | Easy | Wide model selection |
| GitHub Copilot | Fast | Subscription | Easy | Copilot ecosystem |

*opencode manages keys internally - you don't handle them directly

## Configuration Tips

### 1. Use Environment Variables for Secrets

Never put API keys in config files:

```bash
# Good
export OPENROUTER_API_KEY=sk-...

# Bad (in config.yaml)
providers:
  openrouter:
    api_key: sk-...  # Don't do this!
```

### 2. Enable Only What You Need

```yaml
providers:
  opencode:
    enabled: true
  ollama:
    enabled: false  # Disabled until ready
  openrouter:
    enabled: false
```

### 3. Set Reasonable Timeouts

```yaml
providers:
  opencode:
    timeout_seconds: 120  # 2 minutes for slow models
  groq:
    timeout_seconds: 30    # 30 seconds for fast inference
```

### 4. Monitor Health

Enable health checks to detect issues early:

```yaml
providers:
  opencode:
    health_check:
      enabled: true
      interval_seconds: 30
```

## Contributing New Providers

Want to add a provider? See [Contributing Guide](./CONTRIBUTING.md) for:
- Provider adapter trait
- Implementation guidelines
- Testing requirements

---

**Need help with a specific provider?** [Open an issue](https://github.com/sirdavis99/loc-ai-proxy/issues)
