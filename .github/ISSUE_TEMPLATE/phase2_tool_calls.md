[Phase 2] Implement Tool/Function Calling Support

**Feature Category**
- [ ] New Provider Support
- [ ] Streaming Support
- [x] Tool Calls
- [ ] Performance
- [ ] Other

**Description**
Add support for OpenAI-compatible tool/function calling. This allows models to request tool execution and receive results.

**Acceptance Criteria**
- [ ] Support `tools` parameter in chat completions
- [ ] Parse tool calls from provider responses
- [ ] Return tool calls in OpenAI format
- [ ] Handle tool results (as follow-up requests)
- [ ] Support parallel tool calls
- [ ] Tests for tool call flow
- [ ] Documentation and examples

**Technical Notes**

OpenAI tool format:
```json
{
  "tools": [{
    "type": "function",
    "function": {
      "name": "get_weather",
      "description": "Get weather for location",
      "parameters": {
        "type": "object",
        "properties": {
          "location": {"type": "string"}
        },
        "required": ["location"]
      }
    }
  }]
}
```

Response:
```json
{
  "choices": [{
    "message": {
      "role": "assistant",
      "tool_calls": [{
        "id": "call_abc",
        "type": "function",
        "function": {
          "name": "get_weather",
          "arguments": "{\"location\":\"NYC\"}"
        }
      }]
    }
  }]
}
```

**Important:** Pi handles tool execution itself. The proxy just needs to:
1. Forward tool definitions to providers
2. Return tool call requests to Pi
3. Accept tool results in follow-up messages

**Dependencies**
- Requires provider support (opencode supports tools)
- Better to implement after streaming (Phase 2.5)

**Priority**
- [ ] Must Have
- [ ] Should Have
- [x] Nice to Have (Pi handles tools itself)

**Estimated Effort**
4-6 days

**Complexity**: High - requires understanding Pi's tool system
