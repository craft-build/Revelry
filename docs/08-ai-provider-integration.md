<!-- Revelry Design Specification v2.0 — Section 8 -->
<!-- Last updated: 2026-05-20 -->

# AI Provider Integration

## 8.1 rig Framework: Unified Multi-Provider Abstraction

Revelry's provider integration is built on the rig framework, which provides a unified abstraction layer over multiple AI model providers. Unlike the original Tauri design's approach of manually constructing OpenAI-compatible HTTP requests, rig offers purpose-built client implementations for each provider, handling provider-specific authentication, request formatting, SSE streaming, and response parsing. This approach provides several advantages: provider-specific features are accessible through their native API rather than being lost in an OpenAI-compatible translation layer, new providers can be added by configuring the appropriate rig client, and the framework handles the full lifecycle of AI requests including error handling, retries, and token usage tracking.

## 8.2 Supported Providers

| **Provider** | **Type** | **rig Support** | **Notes** |
|---|---|---|---|
| OpenAI | Completion + Embedding | Native client | GPT-4, GPT-3.5, GPT-4o; first-class support |
| Anthropic | Completion + Embedding | Native client | Claude models; native API format |
| Ollama | Completion + Embedding | Native client | Local models; no OpenAI translation needed |
| Gemini | Completion + Embedding | Native client | Google Gemini models |
| Cohere | Completion + Embedding | Native client | Cohere Command models |
| Mistral | Completion + Embedding | Native client | Mistral AI models |
| DeepSeek | Completion + Embedding | Native client | DeepSeek models |
| Groq | Completion + Embedding | Native client | Fast inference via Groq |
| Azure OpenAI | Completion + Embedding | Native client | Azure-hosted OpenAI models |
| OpenRouter | Completion + Embedding | Native client | Multi-provider routing |
| Together AI | Completion + Embedding | Native client | Together AI hosted models |
| HuggingFace | Completion + Embedding | Native client | HF Inference API |
| Perplexity | Completion + Embedding | Native client | Perplexity AI models |
| xAI (Grok) | Completion + Embedding | Native client | Grok models |
| Custom Endpoints | Completion | OpenAI client w/ custom base URL | LM Studio, vLLM, LocalAI, etc. |

## 8.3 Provider Configuration

Each provider is configured with a name, rig client type, API key, and default model selection. Multiple providers can be configured simultaneously, and individual characters or conversations can override the default provider. The configuration is managed through an AdwPreferencesWindow with AdwEntryRow widgets for name and URL fields, AdwPasswordEntryRow for API keys, and AdwComboRow for model selection. API keys are stored in the operating system's native credential store through the keyring crate, never in the SQLite database or configuration files.

Provider-level settings include default generation parameters (temperature, top_p, max_tokens, context length), timeout configuration, and retry policies. Users can create named parameter presets (for example, Creative Writing with temperature 0.9, or Precise Q&A with temperature 0.3) and assign them at the character or conversation level, allowing fine-tuned control over AI behavior without repeatedly adjusting individual parameters.

## 8.4 Streaming Architecture

Streaming is the default response mode for all providers. The rig agent's streaming API returns an async stream of token chunks, which is consumed on the tokio runtime. Each chunk is forwarded to the GLib main loop through a glib::MainContext channel, where the Relm4 model receives it as a StreamToken message and appends it to the active message in the model state. The view! macro reactively updates the GtkLabel or GtkTextView displaying the AI's response, producing a smooth typewriter-style text appearance.

The streaming implementation includes a cancellation mechanism: users can stop generation at any point by clicking a stop button or pressing Escape, which sends a CancelGeneration message. The tokio runtime drops the streaming future, closing the HTTP connection and marking the message as partial. Partial messages are fully functional in the conversation history; they can be regenerated, edited, or branched from just like complete messages. Additionally, the chat-engine module estimates total generated tokens using tiktoken-rs for cost tracking and context management.
