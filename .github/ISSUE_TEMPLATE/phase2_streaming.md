[Phase 2] Implement Streaming Responses (SSE)

**Feature Category**
- [x] New Provider Support
- [x] Streaming Support
- [ ] Tool Calls
- [ ] Performance
- [ ] Other

**Description**
Implement Server-Sent Events (SSE) support for streaming chat completions. This will allow real-time token-by-token responses instead of buffering the entire response.

**Acceptance Criteria**
- [ ] Support `stream: true` parameter in `/v1/chat/completions`
- [ ] Implement SSE response format
- [ ] Buffer provider responses and stream tokens
- [ ] Handle connection drops gracefully
- [ ] Compatible with Pi's streaming expectations
- [ ] Tests for streaming functionality

**Technical Notes**

OpenAI SSE format:
```
data: {"id": "...", "object": "chat.completion.chunk", "choices": [{"delta": {"content": "Hello"}}]}

data: {"id": "...", "object": "chat.completion.chunk", "choices": [{"delta": {"content": " world"}}]}

data: [DONE]
```

Implementation approach:
1. Add `stream` field to `ChatCompletionRequest`
2. Create SSE response type
3. Modify `chat_completions` handler to support both modes
4. Use `axum::response::Sse` for streaming
5. Buffer opencode response and yield tokens

**Dependencies**
- None (can be done independently)

**Priority**
- [x] Must Have (commonly requested feature)
- [ ] Should Have
- [ ] Nice to Have

**Estimated Effort**
3-5 days
