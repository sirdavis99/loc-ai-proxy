# Architecture Design Document

## System Overview

`loc-ai-proxy` is a bridge service that translates between OpenAI-compatible API requests and various AI provider backends.

## Core Principles

1. **OpenAI Compatibility**: Expose standard `/v1/chat/completions` and `/v1/models` endpoints
2. **Provider Agnostic**: Pluggable adapter system for different backends
3. **Session-Based**: Maintain conversation context through session management
4. **Zero-Config**: Auto-detect installed tools and configure automatically
5. **Lightweight**: Minimal resource usage, fast startup

## Data Flow

### Chat Completion Request

```
1. Pi sends POST /v1/chat/completions
   Headers: Authorization: Bearer <token>
   Body: {
     "model": "opencode/anthropic/claude-3.5-sonnet",
     "messages": [{"role": "user", "content": "Hello"}]
   }

2. API Layer receives request
   - Validates OpenAI format
   - Extracts model ID and conversation ID

3. Session Manager checks cache
   - If conversation exists → use existing session
   - If new → create new opencode session

4. Provider Adapter converts request
   - OpenAI format → Provider-specific format
   - Maps model names (e.g., claude-3.5-sonnet)

5. Provider Adapter sends to backend
   - HTTP request to opencode server
   - Or CLI call if HTTP unavailable

6. Response flows back
   - Provider response → OpenAI format
   - Return to Pi

7. Session Manager updates
   - Stores conversation state
   - Sets TTL for cleanup
```

## Component Details

### 1. API Layer (`src/api/`)

**Responsibilities:**
- HTTP server setup (axum)
- Request/response validation
- OpenAI format handling
- CORS and security headers

**Endpoints:**
```rust
POST /v1/chat/completions    # Main chat endpoint
GET  /v1/models              # List available models
GET  /health                 # Health check
```

**Key Structures:**
```rust
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    temperature: Option<f32>,
    max_tokens: Option<i32>,
    stream: Option<bool>,
    conversation_id: Option<String>, // Pi-specific
}

struct ChatCompletionResponse {
    id: String,
    object: String,
    created: i64,
    model: String,
    choices: Vec<Choice>,
    usage: Usage,
}
```

### 2. Session Manager (`src/session/`)

**Responsibilities:**
- Map conversation IDs to provider sessions
- Manage session lifecycle
- Handle cleanup and expiration
- Thread-safe access

**Design:**
```rust
use dashmap::DashMap;
use chrono::{DateTime, Utc};

struct SessionCache {
    sessions: DashMap<String, SessionInfo>,
}

struct SessionInfo {
    provider_session_id: String,
    provider_type: ProviderType,
    last_accessed: DateTime<Utc>,
    created_at: DateTime<Utc>,
    ttl_seconds: i64,
}

impl SessionManager {
    async fn get_or_create_session(
        &self,
        conversation_id: &str,
        provider: ProviderType,
    ) -> Result<String> {
        // Check cache
        if let Some(session) = self.cache.get(conversation_id) {
            if !session.is_expired() {
                session.touch();
                return Ok(session.provider_session_id.clone());
            }
        }
        
        // Create new session
        let provider_session = provider.create_session().await?;
        self.cache.insert(conversation_id, SessionInfo::new(provider_session));
        
        Ok(provider_session)
    }
}
```

**Cleanup Strategy:**
- Background task runs every 5 minutes
- Removes expired sessions
- Also cleans up provider-side sessions if API allows

### 3. Provider Adapter System (`src/providers/`)

**Responsibilities:**
- Abstract different provider implementations
- Handle authentication
- Convert between formats
- Manage provider-specific quirks

**Trait Definition:**
```rust
#[async_trait]
trait ProviderAdapter: Send + Sync {
    fn name(&self) -> &'static str;
    
    async fn is_available(&self) -> bool;
    
    async fn list_models(&self) -> Result<Vec<Model>>;
    
    async fn create_session(&self) -> Result<String>;
    
    async fn send_message(
        &self,
        session_id: &str,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse>;
    
    async fn close_session(&self, session_id: &str) -> Result<()>;
}
```

**Provider Implementations:**

#### opencode Provider (`src/providers/opencode/`)

**Connection Methods:**
1. **HTTP API** (preferred) - Connect to running opencode server
2. **CLI Fallback** - Use `opencode -p` for simple requests

**Session Management:**
- Uses opencode's native session API
- POST `/session` to create
- POST `/session/:id/prompt` to send messages

**Configuration:**
```rust
struct OpencodeConfig {
    url: String,           // http://127.0.0.1:4096
    auto_start: bool,      // Start opencode if not running
    timeout_seconds: u64,
}
```

**Auto-Detection:**
```rust
impl OpencodeProvider {
    async fn detect() -> Option<Self> {
        // Check if opencode in PATH
        if which::which("opencode").is_err() {
            return None;
        }
        
        // Try connecting to default port
        if Self::test_connection("http://127.0.0.1:4096").await {
            return Some(Self::new("http://127.0.0.1:4096"));
        }
        
        // Try to start opencode
        if auto_start_enabled {
            Self::start_opencode().await.ok()?;
            // Retry connection
        }
        
        None
    }
}
```

### 4. Model Registry (`src/models/`)

**Responsibilities:**
- Map model names between formats
- Cache available models from providers
- Handle model aliases

**Model ID Format:**
```
[provider]/[vendor]/[model-id]

Examples:
  opencode/anthropic/claude-3.5-sonnet
  opencode/google/gemini-2.5-pro
  openrouter/anthropic/claude-3-opus
  ollama/llama3.3:70b
```

**Alias System:**
```rust
lazy_static! {
    static ref MODEL_ALIASES: HashMap<&str, &str> = {
        let mut m = HashMap::new();
        m.insert("claude-3.5-sonnet", "opencode/anthropic/claude-3.5-sonnet");
        m.insert("gemini-pro", "opencode/google/gemini-2.5-pro");
        m
    };
}
```

**Discovery:**
- On startup, query each provider for available models
- Cache results with TTL
- Expose via `/v1/models` endpoint

### 5. CLI Interface (`src/cli/`)

**Commands:**
```bash
locaiproxy                          # Start server
locaiproxy configure                # Interactive configuration
locaiproxy configure opencode       # Configure opencode provider
locaiproxy status                   # Show provider status
locaiproxy list-models              # List available models
```

**Configuration File:**
```yaml
# ~/.config/loc-ai-proxy/config.yaml
server:
  port: 9110
  host: 127.0.0.1
  
providers:
  opencode:
    enabled: true
    url: http://127.0.0.1:4096
    auto_start: true
    
logging:
  level: info
  format: json
```

## Error Handling Strategy

### Error Types

```rust
#[derive(Error, Debug)]
enum ProxyError {
    #[error("Provider unavailable: {0}")]
    ProviderUnavailable(String),
    
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    
    #[error("Invalid model: {0}")]
    InvalidModel(String),
    
    #[error("Provider error: {0}")]
    ProviderError(String),
    
    #[error("Request timeout")]
    Timeout,
}
```

### HTTP Status Mapping

| Proxy Error | HTTP Status | OpenAI Format |
|------------|-------------|---------------|
| ProviderUnavailable | 503 | `{"error": {"message": "...", "type": "provider_error"}}` |
| SessionNotFound | 404 | `{"error": {"message": "...", "type": "invalid_request"}}` |
| InvalidModel | 400 | `{"error": {"message": "...", "type": "invalid_model"}}` |
| ProviderError | 502 | `{"error": {"message": "...", "type": "provider_error"}}` |
| Timeout | 504 | `{"error": {"message": "...", "type": "timeout"}}` |

## Security Considerations

1. **Authentication**
   - API key validation (optional)
   - Bearer token support
   - Per-provider credentials

2. **Rate Limiting**
   - Per-client limits
   - Per-provider limits
   - Exponential backoff

3. **Input Validation**
   - Request size limits
   - Content-type validation
   - Model ID sanitization

4. **Logging**
   - Never log API keys
   - Sanitize messages in logs
   - Structured logging (JSON)

## Performance Targets

- **Cold Start**: < 500ms
- **Request Latency**: < 50ms overhead (excluding provider time)
- **Concurrent Connections**: 100+
- **Memory Usage**: < 100MB
- **Session Cleanup**: Automatic, no manual intervention

## Configuration Precedence

1. CLI flags (highest priority)
2. Environment variables
3. Config file
4. Auto-detected defaults (lowest priority)

## Future Extensions

### Phase 2: Additional Providers
- ollama: Direct local model access
- openrouter: Aggregated API
- github-copilot: Copilot models

### Phase 3: Advanced Features
- Streaming responses (SSE)
- Tool call support
- Multi-modal (images)
- Request queueing
- Load balancing across providers

### Phase 4: Enterprise
- Metrics (Prometheus)
- Admin dashboard
- API key management
- Usage analytics

## Development Guidelines

### Code Organization
```
src/
├── main.rs              # Entry point
├── lib.rs               # Library exports
├── config.rs            # Configuration management
├── cli.rs               # CLI argument parsing
├── server.rs            # HTTP server setup
├── api/                 # OpenAI-compatible API
│   ├── mod.rs
│   ├── chat.rs          # /v1/chat/completions
│   ├── models.rs        # /v1/models
│   └── health.rs        # /health
├── providers/           # Provider adapters
│   ├── mod.rs
│   ├── trait.rs         # ProviderAdapter trait
│   ├── opencode/        # opencode implementation
│   │   ├── mod.rs
│   │   ├── client.rs
│   │   └── session.rs
│   └── [future]/
├── session/             # Session management
│   ├── mod.rs
│   ├── cache.rs
│   └── manager.rs
├── models/              # Model registry
│   ├── mod.rs
│   ├── registry.rs
│   └── mapping.rs
└── utils/               # Utilities
    ├── mod.rs
    ├── errors.rs
    └── logging.rs
```

### Testing Strategy

1. **Unit Tests**: Individual components
2. **Integration Tests**: Full request/response cycles
3. **Provider Mock Tests**: Test without real providers
4. **E2E Tests**: With real providers (CI only)

---

*This document evolves with the project. Last updated: 2026-03-11*
