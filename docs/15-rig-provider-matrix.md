<!-- Revelry Design Specification v2.0 — Section 15 -->
<!-- Last updated: 2026-05-20 -->

# Appendix A: rig Provider Compatibility Matrix

The following matrix documents the rig provider clients available for Revelry, along with their capabilities and configuration notes. All providers support streaming completions through rig's unified API. The OpenAI client with custom base URL provides compatibility with any OpenAI-compatible endpoint, including LM Studio, vLLM, LocalAI, and similar self-hosted solutions.

| **Provider** | **Completion** | **Embedding** | **Streaming** | **Custom Base URL** | **Notes** |
|---|---|---|---|---|---|
| OpenAI | Yes | Yes | Yes | Yes | GPT-4, GPT-3.5, GPT-4o; primary reference client |
| Anthropic | Yes | Yes | Yes | N/A | Claude models; native Anthropic API format |
| Ollama | Yes | Yes | Yes | Default localhost | Local models; zero-config for local inference |
| Gemini | Yes | Yes | Yes | N/A | Google Gemini models |
| Cohere | Yes | Yes | Yes | N/A | Cohere Command models |
| Mistral | Yes | Yes | Yes | N/A | Mistral AI models |
| DeepSeek | Yes | Yes | Yes | N/A | DeepSeek models |
| Groq | Yes | Yes | Yes | N/A | Fast inference via Groq LPU |
| Azure OpenAI | Yes | Yes | Yes | Azure endpoint | Azure-hosted OpenAI models |
| OpenRouter | Yes | Yes | Yes | N/A | Multi-provider routing service |
| Together AI | Yes | Yes | Yes | N/A | Together AI hosted models |
| HuggingFace | Yes | Yes | Yes | N/A | HF Inference API |
| Perplexity | Yes | Yes | Yes | N/A | Perplexity AI models |
| xAI (Grok) | Yes | Yes | Yes | N/A | Grok models |
| Custom (OpenAI-compatible) | Yes | Varies | Yes | Required | LM Studio, vLLM, LocalAI, etc. |
