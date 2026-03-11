[Phase 2] Add Ollama Provider Support

**Feature Category**
- [x] New Provider Support
- [ ] Streaming Support
- [ ] Tool Calls
- [ ] Performance
- [ ] Other

**Description**
Add support for Ollama as a first-class provider. Ollama enables running LLMs locally, providing privacy and zero API costs.

**Acceptance Criteria**
- [ ] Implement `OllamaProvider` adapter
- [ ] Auto-detect Ollama installation (default: http://127.0.0.1:11434)
- [ ] List available local models via `/api/tags`
- [ ] Support chat completions via `/api/chat`
- [ ] Handle Ollama-specific error responses
- [ ] Add configuration support
- [ ] Documentation for Ollama setup

**Technical Notes**

Ollama API endpoints:
- `GET /api/tags` - List models
- `POST /api/chat` - Chat completion

Request format:
```json
{
  "model": "llama3.3",
  "messages": [{"role": "user", "content": "Hello"}],
  "stream": false
}
```

Model naming:
- Use model names as-is from Ollama (e.g., `llama3.3:70b`)
- Prefix with `ollama/` (e.g., `ollama/llama3.3:70b`)

**Dependencies**
- None

**Priority**
- [x] Must Have (high demand for local models)
- [ ] Should Have
- [ ] Nice to Have

**Estimated Effort**
2-3 days
