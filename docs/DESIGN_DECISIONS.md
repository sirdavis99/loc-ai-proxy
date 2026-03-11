# Design Decisions & Rationale

This document explains the key design decisions made during the development of loc-ai-proxy.

## 1. Why Rust?

**Decision:** Use Rust as the primary language

**Rationale:**
- **Performance**: Rust's zero-cost abstractions and no-GC pauses ensure consistent low latency
- **Memory Safety**: Eliminates entire classes of bugs (use-after-free, data races)
- **Async/Await**: First-class async support with tokio ecosystem
- **Type Safety**: Catch errors at compile time, not runtime
- **Ecosystem**: Excellent HTTP libraries (axum, reqwest, hyper)
- **Deployment**: Single binary, easy to distribute

**Alternatives Considered:**
- Go: Good for concurrent services, but GC pauses could affect latency
- Python: Fast to develop, but slower runtime and deployment complexity
- TypeScript/Node: Familiar ecosystem, but single-threaded event loop limitations

## 2. Why Port 9110?

**Decision:** Default to port 9110 instead of common alternatives

**Rationale:**
- **8080**: Too common, likely conflicts with other dev servers
- **3000**: Node.js default, frequently in use
- **5000**: Python Flask default, common on macOS
- **9110**: Unused by common dev tools, memorable (9-1-1-0 as "emergency" AI bridge)
- **Configurable**: Can always be changed via env var or CLI flag

## 3. Session Management Strategy

**Decision:** New opencode session per conversation

**Rationale:**
- **Isolation**: Each Pi conversation gets clean context, no cross-contamination
- **Cleanup**: Sessions auto-expire after 30min, preventing resource leaks
- **Simplicity**: One-to-one mapping, easy to reason about
- **Debugging**: Clear session boundaries in logs

**Alternatives Considered:**
- Single shared session: Faster, but context bleeds between conversations
- Pool of sessions: Complex, premature optimization
- No sessions (stateless): Fastest, but loses conversation context entirely

## 4. Model ID Format

**Decision:** Use `provider/vendor/model-id` format with opencode prefix

**Rationale:**
- **Clarity**: Immediately shows which provider and vendor
- **Consistency**: Matches opencode's internal naming
- **Extensibility**: Easy to add new providers without conflicts
- **Aliases**: Support short names (e.g., `claude-3.5-sonnet` → full path)

**Example Mappings:**
```
User requests: "claude-3.5-sonnet"
Internal ID:   "opencode/anthropic/claude-3.5-sonnet"
Backend call:  provider="anthropic", model="claude-3.5-sonnet"
```

## 5. Non-Streaming First (MVP)

**Decision:** Implement non-streaming chat completions first, streaming in Phase 2

**Rationale:**
- **Complexity**: Streaming requires Server-Sent Events (SSE), connection management
- **Value**: Non-streaming works for 80% of use cases
- **Foundation**: Get core architecture solid before adding complexity
- **Testing**: Easier to test and debug without streaming

**Streaming Plan (Phase 2):**
- Implement SSE endpoint
- Buffer provider responses
- Stream tokens as they arrive
- Handle connection drops gracefully

## 6. Tool Calls: Not in MVP

**Decision:** Skip tool call support in initial release

**Rationale:**
- **Scope**: Tool handling requires significant additional complexity
- **Pi**: Pi manages its own tool execution loop
- **opencode**: opencode's tool system is different from OpenAI's format
- **Complexity**: Would need bidirectional translation of tool schemas

**Future Consideration:**
If needed, could add lightweight tool proxy that:
1. Receives tool call from provider
2. Forwards to Pi's tool system
3. Returns results to provider

## 7. Configuration Approach

**Decision:** Three-tier configuration with auto-detection

**Hierarchy (high to low priority):**
1. CLI flags (explicit user intent)
2. Environment variables (deployment-specific)
3. Config file (user preferences)
4. Auto-detection (sensible defaults)

**Rationale:**
- **Flexibility**: Works for local dev, Docker, and production
- **12-Factor**: Follows 12-factor app principles
- **Zero-Config**: Works out-of-the-box for typical setups
- **Explicit**: User can always override when needed

## 8. Why axum for HTTP Server?

**Decision:** Use axum web framework

**Rationale:**
- **Ecosystem**: Built by Tokio team, integrates perfectly
- **Performance**: Zero-allocation routing, fast compile times
- **Type Safety**: Request handlers are strongly typed
- **Middleware**: Excellent middleware system (CORS, logging, timeouts)
- **Documentation**: Great docs and examples

**Alternatives Considered:**
- actix-web: Good performance, but steeper learning curve
- warp: Functional style, filter-based routing (complex for beginners)
- rocket: Great ergonomics, but compile-time heavy

## 9. dashmap for Session Cache

**Decision:** Use dashmap for concurrent session storage

**Rationale:**
- **Performance**: Lock-free concurrent hashmap
- **Ergonomics**: API similar to std HashMap
- **TTL**: Built-in support for expiration
- **Memory**: Efficient for read-heavy workloads (most requests are reads)

**Alternatives Considered:**
- RwLock<HashMap>: Simpler, but locks could bottleneck under load
- evmap: Good for read-heavy, but more complex API
- moka: Feature-rich cache, but overkill for our use case

## 10. Auto-Detect opencode

**Decision:** Automatically detect and optionally auto-start opencode

**Rationale:**
- **UX**: Zero-config setup for most users
- **Integration**: Seamless experience if opencode already installed
- **Graceful**: Falls back gracefully if detection fails
- **Safety**: Auto-start is opt-in, not forced

**Detection Strategy:**
1. Check if `opencode` in PATH
2. Try connecting to default port (4096)
3. If not running and auto_start enabled, try to start
4. Retry connection
5. Mark provider as unavailable if all fail

## 11. Docker Support from Day 1

**Decision:** Include Docker support in initial release

**Rationale:**
- **Portability**: Easiest way to distribute and run
- **Isolation**: Containerized, no dependency conflicts
- **CI/CD**: Easy to test and deploy
- **Community**: Docker users expect official images

**Implementation:**
- Multi-stage build for small image size
- Alpine-based for minimal footprint
- docker-compose.yml for easy local testing

## 12. Error Handling Strategy

**Decision:** Use thiserror for custom errors, anyhow for context

**Rationale:**
- **Clarity**: thiserror generates clean error types with Display impl
- **Context**: anyhow for adding context without new error types
- **Performance**: Zero-cost error handling
- **Ecosystem**: Standard in Rust community

**Error Categories:**
- ProviderError: Backend service failures
- SessionError: Session management issues
- ValidationError: Invalid requests
- TimeoutError: Request timeouts

## 13. CLI Command Structure

**Decision:** Use subcommand pattern: `locaiproxy <command> [args]`

**Rationale:**
- **Discoverability**: `locaiproxy --help` shows all commands
- **Extensibility**: Easy to add new commands
- **Clarity**: Clear separation between server and configuration
- **Unix**: Follows Unix philosophy (do one thing well)

**Command Examples:**
```bash
locaiproxy                          # Start server (default)
locaiproxy configure opencode       # Configure specific provider
locaiproxy status                   # Show status
locaiproxy list-models             # List models
```

## 14. Why Separate Conversation and Session IDs?

**Decision:** Maintain separate Pi conversation ID and provider session ID

**Rationale:**
- **Decoupling**: Pi doesn't need to know about opencode session IDs
- **Portability**: Could switch to different provider without changing Pi
- **Multi-Provider**: Same conversation could use different providers
- **Debugging**: Clear separation in logs

**Mapping:**
```
Pi conversation_id: "conv_abc123"
         ↓
Session Manager maps → opencode session_id: "session_xyz789"
         ↓
opencode backend manages actual session
```

## 15. Testing Strategy

**Decision:** Three-tier testing approach

**Unit Tests:**
- Test individual functions in isolation
- Mock all external dependencies
- Fast execution (< 1s)

**Integration Tests:**
- Test API endpoints with HTTP client
- Use test containers or mocks for providers
- Medium execution time (< 30s)

**E2E Tests:**
- Full flow with real opencode instance
- Run in CI only (not locally)
- Validate actual responses

**Rationale:**
- **Confidence**: Different levels catch different bugs
- **Speed**: Fast feedback during development
- **Isolation**: Unit tests don't need external services
- **Realism**: E2E tests catch integration issues

## 16. Documentation Strategy

**Decision:** Comprehensive documentation from day 1

**Rationale:**
- **Onboarding**: New users need clear setup instructions
- **API Users**: Pi users need to know how to configure
- **Contributors**: Future developers need architecture docs
- **Maintenance**: Docs prevent knowledge silos

**Documentation Types:**
- README.md: Quick start and overview
- ARCHITECTURE.md: System design
- DESIGN_DECISIONS.md: This file - why we made choices
- API.md: OpenAI-compatible endpoint reference
- PROVIDER_SETUP.md: Configure each provider
- PI_SETUP.md: Pi-specific integration guide
- CONTRIBUTING.md: How to contribute

## 17. License: MIT

**Decision:** MIT License

**Rationale:**
- **Permissive**: Maximum freedom for users
- **Commercial**: Can be used in commercial projects
- **Community**: Encourages contributions
- **Standard**: Most widely understood open-source license

## 18. Versioning: SemVer

**Decision:** Follow Semantic Versioning (SemVer)

**Format:** MAJOR.MINOR.PATCH (e.g., 1.2.3)

**Rationale:**
- **MAJOR**: Breaking API changes
- **MINOR**: New features, backwards compatible
- **PATCH**: Bug fixes, backwards compatible

**Pre-1.0:**
- While in development, MINOR changes may break
- Post 1.0, strict SemVer compliance

## Open Questions / Future Decisions

1. **Authentication**: Should we add API key management for multi-user scenarios?
2. **Metrics**: Prometheus metrics for monitoring?
3. **Admin Dashboard**: Web UI for configuration and monitoring?
4. **Load Balancing**: Support for multiple opencode instances?
5. **Caching**: Cache model lists and other frequently accessed data?
6. **WebSocket**: Alternative to SSE for streaming?

---

*Document evolves with the project. Contributions to this document welcome!*
