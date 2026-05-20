<!-- Revelry Design Specification v2.0 — Section 13 -->
<!-- Last updated: 2026-05-20 -->

# Security and Privacy

## 13.1 API Key Security

API keys are the most sensitive credential in the application. Keys are stored exclusively in the operating system's native credential store through the keyring crate: GNOME Keyring (via Secret Service API) on Linux, Keychain Services on macOS, and Credential Manager on Windows. They are never written to the SQLite database, configuration files, or logs. The Rust code loads keys into memory only when needed for a rig API request and drops them immediately after use. This approach is more secure than the Tauri design's approach of passing keys across the IPC bridge, which requires the key to exist in both the Rust and JavaScript memory spaces.

## 13.2 Network Security

All outbound HTTP requests use HTTPS by default, with certificate validation enabled through reqwest's default TLS configuration. Users can optionally disable certificate validation for self-signed local endpoints (for example, local Ollama instances), but this requires an explicit opt-in with a security warning. The application does not make any network requests beyond the user-configured AI provider endpoints. There is no telemetry, analytics, crash reporting, or update checking unless the user explicitly enables these features.

## 13.3 Content Security

Revelry does not implement content filtering or moderation. The application is designed as a creative tool, and users have full control over what they create and how they interact with AI characters. For users who require additional privacy, Vault Mode (Tier 3) provides AES-256-GCM encryption of the database and character files at rest, using Argon2id for key derivation from the user's passphrase.

## 13.4 Input Sanitization

Imported character cards and conversation data undergo sanitization to prevent common attack vectors. JSON parsing uses strict schema validation through serde to reject malformed or oversized payloads. PNG card extraction validates tEXt chunk sizes and rejects cards exceeding reasonable limits. Regex scripts are validated for ReDoS patterns before being applied.
