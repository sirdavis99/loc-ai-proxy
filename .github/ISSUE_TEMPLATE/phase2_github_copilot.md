[Phase 2] Add GitHub Copilot Provider Support

**Feature Category**
- [x] New Provider Support
- [ ] Streaming Support
- [ ] Tool Calls
- [ ] Performance
- [ ] Other

**Description**
Add support for GitHub Copilot models, providing access to GPT-4, Claude, and Gemini models through GitHub's Copilot API.

**Acceptance Criteria**
- [ ] Implement `CopilotProvider` adapter
- [ ] Support GitHub token authentication
- [ ] Auto-detect token from VS Code, Neovim, or `gh` CLI
- [ ] List available Copilot models
- [ ] Support chat completions
- [ ] Handle Copilot-specific rate limits
- [ ] Documentation for setup

**Technical Notes**

Authentication:
- Read from `~/.config/github-copilot/hosts.json`
- Or from environment: `GITHUB_TOKEN`
- Or from config: `providers.copilot.api_key`

Model naming:
- `github-copilot/gpt-4`
- `github-copilot/claude-sonnet`
- `github-copilot/gemini-2.5-pro`

**Dependencies**
- None

**Priority**
- [ ] Must Have
- [x] Should Have (popular request)
- [ ] Nice to Have

**Estimated Effort**
3-4 days

**Additional Context**
Copilot support is experimental in opencode. Direct support gives more control.
