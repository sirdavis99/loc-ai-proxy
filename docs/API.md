# API Reference

OpenAI-compatible API endpoints provided by loc-ai-proxy.

## Base URL

```
http://localhost:9110/v1
```

## Authentication

No authentication required for local usage. Future versions may add API key support.

## Endpoints

### POST /v1/chat/completions

Create a chat completion.

#### Request

```json
{
  "model": "opencode/anthropic/claude-3.5-sonnet",
  "messages": [
    {"role": "user", "content": "Hello, how are you?"}
  ],
  "temperature": 0.7,
  "max_tokens": 1000
}
```

#### Parameters

| Name | Type | Required | Description |
|------|------|----------|-------------|
| model | string | Yes | Model ID (e.g., `opencode/anthropic/claude-3.5-sonnet`) |
| messages | array | Yes | Array of message objects |
| temperature | number | No | Sampling temperature (0-2, default: 1) |
| max_tokens | integer | No | Maximum tokens to generate |
| top_p | number | No | Nucleus sampling (0-1) |
| stream | boolean | No | **Not yet supported** (Phase 2) |
| conversation_id | string | No | Pi-specific: maintains session context |

#### Response

```json
{
  "id": "msg_abc123",
  "object": "chat.completion",
  "created": 1700000000,
  "model": "opencode/anthropic/claude-3.5-sonnet",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "I'm doing well, thank you for asking! How can I help you today?"
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 15,
    "total_tokens": 25
  }
}
```

#### Example (cURL)

```bash
curl -X POST http://localhost:9110/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "opencode/anthropic/claude-3.5-sonnet",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

### GET /v1/models

List available models.

#### Response

```json
{
  "object": "list",
  "data": [
    {
      "id": "opencode/anthropic/claude-3.5-sonnet",
      "object": "model",
      "created": 0,
      "owned_by": "anthropic"
    },
    {
      "id": "opencode/google/gemini-2.5-pro",
      "object": "model",
      "created": 0,
      "owned_by": "google"
    }
  ]
}
```

#### Example (cURL)

```bash
curl http://localhost:9110/v1/models
```

### GET /health

Health check endpoint.

#### Response

```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

## Model IDs

### Format

```
[provider]/[vendor]/[model-id]
```

### Examples

| Model ID | Provider | Description |
|----------|----------|-------------|
| `opencode/anthropic/claude-3.5-sonnet` | opencode | Claude 3.5 Sonnet |
| `opencode/google/gemini-2.5-pro` | opencode | Gemini 2.5 Pro |
| `opencode/groq/llama-3.3-70b` | opencode | Llama 3.3 via Groq |
| `opencode/mistral/mistral-large` | opencode | Mistral Large |

### Aliases

Short aliases are automatically expanded:

| Alias | Expands To |
|-------|-----------|
| `claude-3.5-sonnet` | `opencode/anthropic/claude-3.5-sonnet` |
| `gemini-pro` | `opencode/google/gemini-2.5-pro` |
| `llama-3.3-70b` | `opencode/groq/llama-3.3-70b` |

## Error Responses

All errors follow OpenAI format:

```json
{
  "error": {
    "message": "Description of the error",
    "type": "error_type",
    "code": 400
  }
}
```

### Error Types

| Type | HTTP Status | Description |
|------|-------------|-------------|
| `invalid_request` | 400 | Malformed request |
| `invalid_model` | 400 | Model doesn't exist |
| `session_not_found` | 404 | Session expired or invalid |
| `provider_unavailable` | 503 | Provider not connected |
| `provider_error` | 502 | Provider returned error |
| `timeout` | 504 | Request timed out |
| `internal_error` | 500 | Internal server error |

## Limitations

Current version limitations (to be addressed in Phase 2):

- ❌ Streaming not supported (SSE)
- ❌ Function calling not supported
- ❌ Token counts may be estimates
- ❌ No authentication/authorization

## Pi-Specific Fields

Pi adds a `conversation_id` field to maintain session context:

```json
{
  "model": "opencode/anthropic/claude-3.5-sonnet",
  "messages": [...],
  "conversation_id": "conv_abc123"
}
```

Same `conversation_id` = Same session (maintains context)  
Different `conversation_id` = New session (fresh context)

---

*Note: This API is OpenAI-compatible for easy integration. Some features may differ from OpenAI's implementation.*
