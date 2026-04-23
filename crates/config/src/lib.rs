enum ModelProvider {
    OPENAI,
    ANTHROPIC,
    GEMINI,
    LOCAL
}

struct LLMConfig {
    model_provider: ModelProvider,
    model_name: String,
    api_key: String,
}