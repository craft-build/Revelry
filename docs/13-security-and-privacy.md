<!-- Revelry Design Specification v2.0 — Section 13 -->
<!-- Last updated: 2026-05-20 -->

# Security and Privacy

## 13.1 API Key Security

API keys are the most sensitive credential in the application. Keys are stored exclusively in the operating system's native credential store through the keyring crate (v4.0): macOS Keychain Services, GNOME Keyring (via Secret Service API) on Linux, and Credential Manager on Windows. They are never written to the SQLite database, configuration files, or logs. The Rust backend loads keys into memory only when needed for a rig API request and drops them immediately after use. The keyring integration requires `use_native_store(true)` initialization and `release_store()` shutdown for clean lifecycle management.

## 13.2 Network Security

All outbound HTTP requests use HTTPS by default, with certificate validation enabled through reqwest's default TLS configuration. Users can optionally disable certificate validation for self-signed local endpoints (for example, local Ollama instances), but this requires an explicit opt-in with a security warning. The application does not make any network requests beyond the user-configured AI provider endpoints. There is no telemetry, analytics, crash reporting, or update checking unless the user explicitly enables these features.

## 13.3 Content Security

Revelry does not implement content filtering or moderation. The application is designed as a creative tool, and users have full control over what they create and how they interact with AI characters. For users who require additional privacy, Vault Mode (Tier 3) provides AES-256-GCM encryption of the database and character files at rest, using Argon2id for key derivation from the user's passphrase.

## 13.4 Input Sanitization

Imported character cards and conversation data undergo sanitization to prevent common attack vectors. JSON parsing uses strict schema validation through serde to reject malformed or oversized payloads. PNG card extraction validates tEXt chunk sizes and rejects cards exceeding reasonable limits. Regex scripts are validated for ReDoS patterns before being applied. The frontend uses DOMPurify to sanitize HTML in markdown-rendered AI responses, preventing XSS from untrusted model output. Tauri's content security policy provides an additional layer of protection for the webview.

## 13.5 Embeddings and Semantic Features

Semantic features (semantic/hybrid lorebook activation, story-memory recall, semantic search) are powered by an optional embedding source configured in Settings. Embeddings are **off by default**; every feature degrades to its non-semantic path (keyword activation, summary-only memory, FTS5 search) when they are disabled or the source fails at runtime. Two source kinds are supported:

- **Provider (remote):** message and lore text is sent to the selected provider's embedding endpoint to compute vectors. The provider's API key stays in the OS keychain; only the resulting vectors are stored locally in SQLite (in the derived, rebuildable `embeddings` table). Users should choose a provider they trust with conversation content, exactly as they already do for chat completions.
- **Local model (fastembed):** vectors are computed on-device by an ONNX model downloaded on first use. Nothing leaves the machine.

The vector index is derived state — it can be deleted or rebuilt at any time without touching canonical data, and rows are keyed by embedding model so switching sources never mixes incompatible vector spaces.
