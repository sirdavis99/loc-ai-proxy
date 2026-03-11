use std::collections::HashMap;
use lazy_static::lazy_static;

lazy_static! {
    static ref MODEL_ALIASES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();

        // Claude models (direct Anthropic)
        m.insert("claude-3-opus", "anthropic/claude-3-opus-20240229");
        m.insert("claude-3-5-sonnet", "anthropic/claude-3-5-sonnet-20241022");
        m.insert("claude-3-5-haiku", "anthropic/claude-3-5-haiku-20241022");

        // Claude models via opencode (fallback)
        m.insert("claude-3.5-sonnet", "opencode/anthropic/claude-3.5-sonnet");
        m.insert("claude-3-opus-via-opencode", "opencode/anthropic/claude-3-opus");
        m.insert("claude-3-haiku-via-opencode", "opencode/anthropic/claude-3-haiku");

        // Gemini models
        m.insert("gemini-pro", "opencode/google/gemini-2.5-pro");
        m.insert("gemini-flash", "opencode/google/gemini-2.5-flash");

        // Llama models
        m.insert("llama-3.3-70b", "opencode/groq/llama-3.3-70b");
        m.insert("llama-3.3", "opencode/meta/llama-3.3-70b");

        // Mistral models
        m.insert("mistral-large", "opencode/mistral/mistral-large");
        m.insert("mistral-medium", "opencode/mistral/mistral-medium");

        // DeepSeek models
        m.insert("deepseek-chat", "opencode/openrouter/deepseek/deepseek-chat");
        m.insert("deepseek-r1", "opencode/openrouter/deepseek/deepseek-r1");

        m
    };
}

pub fn resolve_model_alias(alias: &str) -> String {
    MODEL_ALIASES
        .get(alias)
        .map(|&s| s.to_string())
        .unwrap_or_else(|| alias.to_string())
}

pub fn is_alias(model_id: &str) -> bool {
    MODEL_ALIASES.contains_key(model_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_alias() {
        assert_eq!(
            resolve_model_alias("claude-3-opus"),
            "anthropic/claude-3-opus-20240229"
        );
    }

    #[test]
    fn test_non_alias_returns_original() {
        assert_eq!(
            resolve_model_alias("custom/model"),
            "custom/model"
        );
    }
}
