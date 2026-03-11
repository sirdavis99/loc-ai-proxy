[Phase 2] Add OpenRouter Provider Support

**Feature Category**
- [x] New Provider Support
- [ ] Streaming Support
- [ ] Tool Calls
- [ ] Performance
- [ ] Other

**Description**
Add direct OpenRouter provider support for access to 150+ models without going through opencode. This provides redundancy and potentially better pricing.

**Acceptance Criteria**
- [ ] Implement `OpenRouterProvider` adapter
- [ ] Support OpenRouter API key authentication
- [ ] List models via OpenRouter `/models` endpoint
- [ ] Support chat completions via `/chat/completions`
- [ ] Handle OpenRouter-specific features (free models, routing)
- [ ] Add configuration support
- [ ] Documentation for API key setup

**Technical Notes**

OpenRouter is already OpenAI-compatible:
- Base URL: `https://openrouter.ai/api/v1`
- Same request/response format as OpenAI
- Just need to add `Authorization: Bearer <key>` header

Model naming:
- Use OpenRouter IDs: `openrouter/anthropic/claude-3.5-sonnet`
- Or vendor/model format: `anthropic/claude-3.5-sonnet`

**Dependencies**
- None

**Priority**
- [ ] Must Have
- [x] Should Have (redundancy, direct access)
- [ ] Nice to Have

**Estimated Effort**
2-3 days
