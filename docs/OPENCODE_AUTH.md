# OpenCode Authentication Guide

This document explains how opencode authentication works and how to configure loc-ai-proxy to use it.

## Overview

OpenCode (the server) uses **HTTP Basic Authentication** to secure its API endpoints. This is separate from the provider API keys (OpenRouter, Anthropic, etc.) that opencode uses internally.

## Authentication System

### How It Works

1. When opencode server starts (`opencode serve`), it generates a unique password
2. The credentials are stored in environment variables:
   - `OPENCODE_SERVER_USERNAME=opencode`
   - `OPENCODE_SERVER_PASSWORD=<random-uuid>`
3. All API requests must include these credentials via HTTP Basic Auth

### Credential Storage

The proxy can obtain credentials from:

1. **Environment Variables** (Auto-detection)
   ```bash
   export OPENCODE_SERVER_USERNAME=opencode
   export OPENCODE_SERVER_PASSWORD=f84b8b75-2b6c-40ee-b827-f4c5c3e96987
   ```

2. **Configuration File** (Explicit)
   ```yaml
   providers:
     opencode:
       enabled: true
       type: opencode
       url: http://127.0.0.1:4096
       auth:
         username: opencode
         password: your-password-here
   ```

3. **Provider API Keys** (Separate from server auth)
   - Stored in `~/.local/share/opencode/auth.json`
   - These are for individual providers (OpenRouter, Google, etc.)
   - Not needed for the proxy to connect to opencode server

## Finding Your Credentials

### Method 1: From Running Process

If opencode is already running, check its environment:

```bash
# macOS/Linux
ps aux -E | grep opencode | grep -E "OPENCODE_SERVER"

# Or look at your current shell environment
env | grep OPENCODE_SERVER
```

### Method 2: From Config File

If you previously configured the proxy:

```bash
cat ~/.config/loc-ai-proxy/config.yaml
```

### Method 3: Start Fresh

1. Kill existing opencode server:
   ```bash
   pkill opencode
   ```

2. Start opencode server in foreground to see credentials:
   ```bash
   opencode serve
   ```
   
   Or check environment after starting:
   ```bash
   opencode serve &
   sleep 2
   env | grep OPENCODE_SERVER
   ```

## Configuration Examples

### Example 1: Using Environment Variables (Recommended)

```bash
# Start opencode and capture credentials
opencode serve &
export OPENCODE_SERVER_USERNAME=$(env | grep OPENCODE_SERVER_USERNAME | cut -d= -f2)
export OPENCODE_SERVER_PASSWORD=$(env | grep OPENCODE_SERVER_PASSWORD | cut -d= -f2)

# Start proxy (will auto-detect credentials)
locaiproxy
```

### Example 2: Using Config File

Create `~/.config/loc-ai-proxy/config.yaml`:

```yaml
server:
  port: 9110
  host: 127.0.0.1

providers:
  opencode:
    enabled: true
    type: opencode
    url: http://127.0.0.1:4096
    auth:
      username: opencode
      password: f84b8b75-2b6c-40ee-b827-f4c5c3e96987
```

### Example 3: Using Different Ports

If opencode runs on a non-default port:

```yaml
providers:
  opencode:
    enabled: true
    type: opencode
    url: http://127.0.0.1:59775  # Custom port
    auth:
      username: opencode
      password: your-password
```

## Testing Authentication

### Test 1: Direct API Call

```bash
# Replace with your actual credentials
curl -u "opencode:your-password" \
  http://127.0.0.1:4096/global/health

# Expected response:
# {"healthy":true,"version":"1.2.24"}
```

### Test 2: Via Proxy

```bash
# Start proxy with auth configured
locaiproxy

# Test health endpoint
curl http://localhost:9110/health

# Expected: {"status":"ok","version":"0.1.0"}

# Test models endpoint
curl http://localhost:9110/v1/models
```

### Test 3: Pi Integration

```bash
# In Pi, switch to loc-ai-proxy provider
pi
/provider loc-ai-proxy

# Or test directly
curl -X POST http://localhost:9110/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-3.5-sonnet",
    "messages": [{"role": "user", "content": "Hello"}]
  }'
```

## Troubleshooting

### Error: "Authentication not configured"

**Cause**: Proxy can't find credentials

**Solutions**:
1. Set environment variables:
   ```bash
   export OPENCODE_SERVER_USERNAME=opencode
   export OPENCODE_SERVER_PASSWORD=your-password
   ```

2. Or create config file with auth section

3. Or run proxy from same shell where opencode was started

### Error: "Authentication failed" or 401 Unauthorized

**Cause**: Wrong credentials

**Solutions**:
1. Verify credentials from opencode process:
   ```bash
   ps aux -E | grep opencode
   ```

2. Check that password hasn't changed (opencode generates new one on restart)

3. Ensure you're using the right port (check `opencode serve` output)

### Error: "Connection refused"

**Cause**: opencode server not running

**Solutions**:
```bash
# Check if opencode is running
pgrep -f "opencode serve"

# If not, start it
opencode serve &

# Or enable auto-start in config
```

## Security Notes

1. **Password Rotation**: OpenCode generates a new password each time it starts
2. **Scope**: The server password is only for the local HTTP API, not for cloud services
3. **Provider Keys**: Your actual API keys (OpenRouter, etc.) are stored separately in `~/.local/share/opencode/auth.json`
4. **Local Only**: These credentials only work on localhost, they're not exposed to the internet

## Architecture

```
┌─────────────┐    HTTP Basic Auth    ┌──────────────┐
│ loc-ai-proxy│◄─────────────────────│ opencode     │
│   (9110)    │  username: opencode   │   server     │
└─────────────┘  password: <uuid>     └──────┬───────┘
                                              │
                                              │ Uses provider API keys
                                              ▼
                                    ┌──────────────────┐
                                    │ Provider APIs    │
                                    │ (OpenRouter,     │
                                    │  Anthropic, etc) │
                                    └──────────────────┘
```

## Advanced: Multiple Instances

If running multiple opencode instances:

```yaml
providers:
  opencode-primary:
    enabled: true
    type: opencode
    url: http://127.0.0.1:4096
    auth:
      username: opencode
      password: password-1

  opencode-secondary:
    enabled: true
    type: opencode
    url: http://127.0.0.1:4097
    auth:
      username: opencode
      password: password-2
```

Then in Pi:
```json
{
  "providers": {
    "opencode-primary": { ... },
    "opencode-secondary": { ... }
  }
}
```

## See Also

- [PROVIDER_SETUP.md](./PROVIDER_SETUP.md) - General provider configuration
- [PI_SETUP.md](./PI_SETUP.md) - Pi integration guide
- [ARCHITECTURE.md](./ARCHITECTURE.md) - System design
