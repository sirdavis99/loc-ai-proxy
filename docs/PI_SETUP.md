# Pi Integration Guide

This guide shows you how to configure Pi to use loc-ai-proxy for accessing opencode's providers.

## Overview

By setting up loc-ai-proxy, you can use Pi to access all 300+ models available through opencode, including:
- Anthropic (Claude 3.5 Sonnet, Claude 3 Opus, etc.)
- Google (Gemini 2.5 Pro, Gemini 2.5 Flash, etc.)
- Groq (Llama 3.3 70B, fast inference)
- Mistral (various models)
- NVIDIA (NIM endpoints)
- OpenRouter (150+ models)
- And more...

## Prerequisites

1. **Pi installed** - [Install Pi](https://pi.dev/) if you haven't already
2. **opencode installed** - [Install opencode](https://opencode.ai/)
3. **loc-ai-proxy installed** - Follow the [main README](../README.md)

## Quick Setup

### Step 1: Start loc-ai-proxy

```bash
# Start the proxy (default port 9110)
locaiproxy

# Or configure opencode first
locaiproxy configure opencode
```

Verify it's running:
```bash
curl http://localhost:9110/health
# Expected: {"status": "ok"}
```

### Step 2: Configure Pi

Edit your Pi configuration file at `~/.pi/agent/models.json`:

```bash
# Open the file
pi config  # or edit directly: nano ~/.pi/agent/models.json
```

Add the loc-ai-proxy provider:

```json
{
  "providers": {
    "loc-ai-proxy": {
      "baseUrl": "http://localhost:9110/v1",
      "api": "openai-completions",
      "models": [
        { "id": "opencode/anthropic/claude-3.5-sonnet", "name": "Claude 3.5 Sonnet", "reasoning": true },
        { "id": "opencode/google/gemini-2.5-pro", "name": "Gemini 2.5 Pro" },
        { "id": "opencode/groq/llama-3.3-70b", "name": "Llama 3.3 70B (Groq)" },
        { "id": "opencode/mistral/mistral-large", "name": "Mistral Large" },
        { "id": "opencode/openrouter/deepseek/deepseek-r1", "name": "DeepSeek R1", "reasoning": true }
      ]
    }
  },
  "defaultProvider": "loc-ai-proxy",
  "defaultModel": "opencode/anthropic/claude-3.5-sonnet"
}
```

### Step 3: Switch Provider in Pi

```bash
# Start Pi
pi

# Switch to loc-ai-proxy provider
/provider loc-ai-proxy

# Or switch to specific model
/model loc-ai-proxy opencode/anthropic/claude-3.5-sonnet
```

That's it! Pi will now use loc-ai-proxy to access opencode's providers.

## Advanced Configuration

### Using Model Aliases

Instead of full paths, you can use aliases. Add to your Pi config:

```json
{
  "providers": {
    "loc-ai-proxy": {
      "baseUrl": "http://localhost:9110/v1",
      "api": "openai-completions",
      "models": [
        { "id": "claude-sonnet", "name": "Claude 3.5 Sonnet" },
        { "id": "gemini-pro", "name": "Gemini 2.5 Pro" },
        { "id": "llama-groq", "name": "Llama 3.3 70B" }
      ]
    }
  }
}
```

These aliases map to full model IDs automatically.

### Using with Docker

If running loc-ai-proxy in Docker:

```json
{
  "providers": {
    "loc-ai-proxy": {
      "baseUrl": "http://host.docker.internal:9110/v1",
      "api": "openai-completions",
      "models": [...]
    }
  }
}
```

Note: `host.docker.internal` only works on Docker Desktop. For Linux, use the host IP.

### Multiple Providers

You can keep your existing providers and add loc-ai-proxy:

```json
{
  "providers": {
    "openai": { ... },
    "anthropic": { ... },
    "loc-ai-proxy": {
      "baseUrl": "http://localhost:9110/v1",
      "api": "openai-completions",
      "models": [...]
    }
  }
}
```

Then switch between them with `/provider <name>`.

## Model Discovery

### List Available Models

You can query loc-ai-proxy to see all available models:

```bash
curl http://localhost:9110/v1/models | jq '.data[].id'
```

Or in Pi, models will appear in the model picker dialog (Ctrl+O).

### Dynamic Model List

For automatic model discovery, loc-ai-proxy supports the `/v1/models` endpoint. Configure Pi to use it:

```json
{
  "providers": {
    "loc-ai-proxy": {
      "baseUrl": "http://localhost:9110/v1",
      "api": "openai-completions"
      // Note: models array omitted - Pi will fetch dynamically
    }
  }
}
```

## Troubleshooting

### Proxy Not Responding

**Problem:** Pi shows connection errors

**Check:**
```bash
# Is loc-ai-proxy running?
curl http://localhost:9110/health

# Is port correct?
netstat -an | grep 9110

# Check logs
RUST_LOG=debug locaiproxy
```

**Solution:**
```bash
# Restart loc-ai-proxy
pkill locaiproxy
locaiproxy
```

### opencode Not Connected

**Problem:** loc-ai-proxy can't connect to opencode

**Check:**
```bash
# Is opencode running?
ps aux | grep opencode

# Try connecting directly
curl http://127.0.0.1:4096/global/health
```

**Solution:**
```bash
# Start opencode
opencode serve

# Or configure opencode path
locaiproxy configure opencode --path /usr/local/bin/opencode
```

### Session Errors

**Problem:** "Session not found" or context lost

**Cause:** Sessions expire after 30 minutes of inactivity

**Solution:** Just continue the conversation - a new session will be created automatically

### Model Not Found

**Problem:** Pi says model doesn't exist

**Check:**
```bash
# Verify model exists in opencode
opencode models | grep "claude-3.5-sonnet"

# Check proxy model list
curl http://localhost:9110/v1/models | jq '.data[].id'
```

**Solution:**
- Ensure model ID format is correct: `provider/vendor/model-id`
- Check that opencode has the model available

## Performance Tuning

### For Faster Responses

1. **Use Groq models** - Fastest inference (Llama 3.3 70B)
2. **Keep sessions alive** - Don't let conversations idle for 30+ minutes
3. **Local opencode** - Ensure opencode is running locally, not remote

### For Cost Savings

1. **Use free models** via OpenRouter:
   - `opencode/openrouter/meta-llama/llama-3.3-70b-instruct:free`
   - `opencode/openrouter/qwen/qwen3-coder:free`

2. **Local Ollama models** (Phase 2):
   - No API costs for local inference

## Best Practices

### 1. Set Default Model

Configure your preferred model as default in Pi:

```json
{
  "defaultProvider": "loc-ai-proxy",
  "defaultModel": "opencode/anthropic/claude-3.5-sonnet"
}
```

### 2. Use Reasoning Models Wisely

Models marked with `"reasoning": true` (Claude 3.5 Sonnet, Gemini 2.5 Pro) are great for complex tasks but slower. Use non-reasoning models for simple queries.

### 3. Monitor Usage

Track which models you use most:

```bash
# View loc-ai-proxy logs
tail -f /var/log/loc-ai-proxy/app.log
```

### 4. Keep Configuration in Version Control

Store your `models.json` in dotfiles:

```bash
ln -s ~/.dotfiles/pi-models.json ~/.pi/agent/models.json
```

## Keyboard Shortcuts in Pi

When using loc-ai-proxy:

- `Ctrl+O` - Open model picker (shows all available models)
- `Ctrl+L` - Switch model within current provider
- `/provider loc-ai-proxy` - Switch to loc-ai-proxy
- `/model <model-id>` - Switch to specific model

## FAQ

**Q: Do I need API keys?**
A: No! loc-ai-proxy uses opencode's authentication, which handles all provider keys internally.

**Q: Can I use multiple providers simultaneously?**
A: Yes, but only one per conversation. Switch providers with `/provider` command.

**Q: Does context persist across model switches?**
A: Within the same provider, yes. Switching providers starts a fresh conversation.

**Q: Can I use this with other OpenAI-compatible tools?**
A: Yes! Any tool that speaks OpenAI API can use loc-ai-proxy.

**Q: What about streaming responses?**
A: Coming in Phase 2. For now, responses are buffered and returned complete.

## Next Steps

- Explore available models: `curl http://localhost:9110/v1/models`
- Try different providers for different tasks
- Configure your favorite models as shortcuts
- Check [Provider Setup](./PROVIDER_SETUP.md) for advanced configuration

## Getting Help

- [GitHub Issues](https://github.com/sirdavis99/loc-ai-proxy/issues)
- [Pi Discord](https://discord.gg/3cU7Bz4UPx)
- [opencode Discord](https://discord.gg/nKXTsAcmbT)

---

**Happy coding with your expanded model access! 🚀**
