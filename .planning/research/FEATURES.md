# Feature Landscape

**Domain:** AI Roleplay Desktop Application (local-first, privacy-focused, multi-provider)
**Researched:** 2026-04-28

## Table Stakes

Features users expect. Missing = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Multi-provider AI connection | Every competitor supports multiple backends. Single-provider = non-starter. SillyTavern supports 15+ APIs, RisuAI supports 10+, Agnai supports 8+. | High | rig crate gives Revelry 20+ providers for free. Must stream tokens. |
| Character card CRUD | Core concept of the entire genre. Users create, edit, delete character definitions. | Med | CharacterCardV2 spec is the de facto standard. Must support all V2 fields. |
| Character card import/export (PNG + JSON) | SillyTavern/PNG spec is the interchange format. Entire sharing ecosystem runs on PNG cards with embedded metadata. | Med | PNG import requires extracting tEXt chunks. Must validate schema. |
| Character library with search & tags | Users accumulate dozens to hundreds of characters. Unsearchable list = unusable. | Low | Grid view with tag filters is standard. SillyTavern, RisuAI, Agnai all do this. |
| Conversation history with message CRUD | Basic expectation: read, edit, delete any message in history. | Med | Every platform supports this. |
| Conversation branching (swipes) | Swipe to regenerate any message, creating branches. SillyTavern calls them "swipes". This is THE core interaction pattern for roleplay. | Med | Must support branch navigation with visual indicators. SillyTavern supports unlimited branches per message. |
| Streaming responses | Token-by-token streaming with typewriter rendering. Non-streaming feels broken in 2026. | Med | Must support cancellation mid-stream. |
| User personas | SillyTavern, RisuAI, Agnai all support user identity definitions (name, description, avatar) that inject into prompts. | Med | Persona locking (per-chat, per-character, default) is expected. |
| Template variable resolution | `{{user}}`, `{{char}}` macros are universal across all platforms. Missing = cards from ecosystem break. | Low | Must support at minimum: `{{user}}`, `{{char}}`, plus persona fields. SillyTavern has 100+ macros. |
| Lorebook/World Info | Keyword-activated dynamic prompt injection. Present in SillyTavern, RisuAI, Agnai. The single most powerful prompt engineering tool in roleplay. | High | Must support: keys, content, insertion position/order, scan depth, recursive activation, token budgeting. SillyTavern's WI is extremely feature-rich. |
| Token counting & context budget | Users MUST know how many tokens their prompt consumes and manage context. | Med | tiktoken-rs planned. Must show permanent vs. dynamic token allocation. |
| Generation parameter presets | Temperature, top_p, max_tokens, frequency/presence penalties, stop sequences. Every platform supports per-generation tuning. | Low | Must be assignable at character and conversation levels. |
| Markdown rendering | AI responses contain `*actions*`, `**emphasis**`, headers, lists. Plain text = unreadable. | Low | pulldown-cmark + Pango markup planned. |
| Local data storage | SillyTavern, RisuAI are both local-first. Cloud-only = privacy dealbreaker for this audience. | Low | SQLite already planned. No cloud dependency. |
| No content filtering | This audience explicitly rejects content moderation. Character.ai loses users specifically because of its filter. SpicyChat markets "no filter" as primary feature. | N/A | Already in project scope. Non-negotiable for target audience. |

## Differentiators

Features that set product apart. Not expected, but valued.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Native GTK4/libadwaita desktop UI | Every competitor is web-based (Node.js server + browser, or Electron/Tauri). A native GNOME desktop app is genuinely unique. Lower latency, smaller binary, proper OS integration. | Very High | This IS the core differentiator. SillyTavern is a Node.js web app. RisuAI uses Tauri (web frontend). Agnai is a web app. Nobody does native desktop. |
| Pure Rust single-process architecture | No IPC bridge, no Node.js server, no web context. Simpler mental model, lower latency, better security surface. | Very High | SillyTavern requires a Node.js server process. RisuAI needs Tauri IPC. Revelry is one binary. |
| OS-native credential storage | API keys in GNOME Keyring/macOS Keychain/Windows Credential Manager. SillyTavern stores keys in browser localStorage. | Low | keyring crate makes this straightforward. Significant security advantage. |
| Structured persona system | Revelry's personas have structured fields (appearance, personality, background, role, relationships, speech style) vs. SillyTavern's flat text blob. Enables persona-aware lorebook activation and token budget compression. | Med | SillyTavern personas are name + avatar + free-text description. Revelry's structured approach is genuinely more powerful. |
| Persona token compression | Full → Trimmed → Minimal → Essential tiered compression when token budget is tight. No competitor does this. | Med | Unique feature. Addresses real pain point: persona tokens eating context. |
| Child persona inheritance | Create personas that inherit from parents with field-level fallback. Enables character family trees, alternate versions. | Med | No competitor has this. Unique organizational tool for power users. |
| Dual-persona mode | Portray two identities in a single conversation. No competitor supports this natively. | Med | Niche but powerful for specific roleplay scenarios. |
| Persona evolution tracking | Timestamped snapshots linked to conversations. See how a persona changed over time. | Med | No competitor tracks persona history. Unique for long-running campaigns. |
| Persona-aware lorebook activation | Lorebook entries activate based on active persona data, not just chat keywords. | Low | SillyTavern added persona description matching as a WI source recently. Revelry's structured personas make this more precise. |
| AES-256-GCM vault encryption | Encrypt database and character files with Argon2id key derivation. | Med | No competitor offers local encryption. Privacy-conscious users value this. SillyTavern data is plaintext JSON/JSONL. |
| Regex find-and-replace scripts | Per-character regex with ReDoS validation. SillyTavern and RisuAI both have this. | Med | Not a differentiator from top competitors, but expected. Must include ReDoS protection. |
| FTS5 full-text search across conversations | SQLite FTS5 for searching all chat history. SillyTavern has basic chat search but not FTS across all conversations. | Low | Practical differentiator. Users with hundreds of chats need this. |
| Guided character creation wizard | Multi-step wizard (AdwPreferencesWindow) for creating characters. SillyTavern's creation is a flat form. | Low | Better UX for newcomers. Progressive disclosure. |
| Import from SillyTavern & CharacterAI formats | Direct import paths from the two most popular ecosystems. | Med | Critical for adoption. Must be seamless. |
| Batch import/export | Import/export entire character libraries at once. | Low | SillyTavern supports this. Must match. |
| Backup/restore as portable archive | SQL dump or ZIP archive. Portability between machines. | Low | SillyTavern has backup. Must match. |
| Validated import with security checks | Schema validation, PNG chunk size limits, ReDoS detection for regex scripts. | Low | Security-conscious. SillyTavern does not validate imports this thoroughly. |

## Anti-Features

Features to explicitly NOT build.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Cloud synchronization | Contradicts local-first, privacy-first core value. Adds server infrastructure cost and complexity. Character.ai and SpicyChat are cloud-only — that's their space, not ours. | Local-only SQLite. Backup/restore for manual portability. |
| Online account system | Same as cloud sync. No accounts = no tracking = no data leaks. | No auth. Settings and data live locally. |
| Built-in content moderation/filtering | Target audience explicitly rejects this. Character.ai loses users to SillyTavern because of filters. Adding filtering would alienate the core user base. | No filtering. Trust users. |
| Web-based UI (Electron/Tauri/WebView) | Defeats the core technical differentiator. Web UI is the one thing every competitor already does. | GTK4/libadwaita native widgets. |
| Built-in image generation | SillyTavern has SD/FLUX/DALL-E integration. RisuAI has image generation. These require separate server infrastructure or API costs. Core competency is text roleplay. | Defer indefinitely. If demanded, add as extension/plugin later. |
| Built-in TTS | SillyTavern has 15+ TTS providers. Complex integration surface. Not core to text roleplay. | Defer. Audio is a different product dimension. |
| Built-in RAG/vector storage | SillyTavern's Data Bank is powerful but complex. Embedding models, chunking, vector storage — huge scope. | Defer. The lorebook system covers most knowledge injection needs. |
| Extension/plugin system | SillyTavern's extension system is its most complex subsystem. Building a plugin API for a Rust/GTK4 app is an enormous undertaking. | Ship core features solidly. Extension API is a v2+ concern. |
| Character expression sprites | SillyTavern and RisuAI both support emotion-based character images. Requires sentiment classification model + image management. Complex for GTK4 (no CSS-based sprite overlay like web). | Defer. Not core to text roleplay. |
| Visual Novel mode | SillyTavern's VN mode is web-specific (full-screen sprite + text overlay). GTK4 adaptation would be a completely different UI paradigm. | Defer. Not aligned with GNOME desktop patterns. |
| Chat translation | SillyTavern has auto-translate. Requires translation API integration. | Defer. Not core. Users can use OS-level translation. |
| Web search integration | SillyTavern has web search extension. Requires search API keys. | Defer. Not core to roleplay. |
| STscript/scripting engine | SillyTavern has a full scripting language (STscript). Enormous scope. | Defer indefinitely. Slash commands for basic operations are enough. |
| Multi-user/collaborative chat | Agnai supports multiple human users in one chat. Requires server infrastructure, auth, real-time sync. | Never. Contradicts local-first architecture. |
| Subscription/monetization | Agnai has subscriptions. SillyTavern is free. This is a personal hobby project. | Free and open source. No payment infrastructure. |
| Mobile support | The project explicitly scopes out mobile. GTK4 on mobile is immature. | Desktop-first. Mobile is a separate future project. |
| Social features (character marketplace, sharing) | Character.ai and Chub have social sharing. Requires cloud infrastructure. | Local import/export. Community can share cards via files. |

## Feature Dependencies

```
Character Card CRUD → Character Import/Export (need CRUD first)
Character Card CRUD → Character Library (need cards to browse)
Character Card CRUD → Lorebook (lorebooks are attached to characters)
Character Card CRUD → User Personas (personas are separate but interact with cards)
Character Card CRUD → Group Chat (group chat requires multiple character cards)
Template Variables → Character Cards (cards use {{user}}, {{char}})
Template Variables → User Personas (personas use {{persona.*}})
Streaming → Multi-provider Connection (streaming is per-provider)
Token Counting → Context Budget Management (need counting for budgeting)
Token Counting → Persona Compression (need counting to decide when to compress)
Lorebook → Token Counting (lorebook budget is token-based)
Conversation Branching → Conversation History (branching extends basic history)
OS Credential Storage → Multi-provider Connection (keys needed for providers)
Regex Scripts → Character Cards (scripts are per-character)
FTS5 Search → Conversation History (search operates on history)
Backup/Restore → All data features (needs access to all data)
```

## Feature Presence Across Competitors

| Feature | SillyTavern | RisuAI | Agnai | Character.ai | SpicyChat |
|---------|:-----------:|:------:|:-----:|:------------:|:---------:|
| Multi-provider | ✅ 15+ | ✅ 10+ | ✅ 8+ | ❌ (own only) | ❌ (own only) |
| CharacterCardV2 | ✅ | ✅ | ✅ | ❌ (proprietary) | ❌ (proprietary) |
| PNG card import | ✅ | ✅ | ✅ | ❌ | ❌ |
| Lorebook/World Info | ✅ (very rich) | ✅ | ✅ | ❌ | ✅ (basic) |
| User personas | ✅ | ✅ | ✅ | ❌ | ✅ (basic) |
| Template macros | ✅ 100+ | ✅ | ✅ (basic) | ❌ | ❌ |
| Conversation branching | ✅ (swipes) | ✅ | ✅ | ✅ (regenerate) | ✅ (basic) |
| Group chat | ✅ | ✅ | ✅ (multi-user) | ✅ | ✅ (basic) |
| Streaming | ✅ | ✅ | ✅ | ✅ | ✅ |
| Token counting | ✅ | ✅ | ✅ | ❌ | ❌ |
| Regex scripts | ✅ | ✅ | ❌ | ❌ | ❌ |
| Character sprites | ✅ | ✅ | ❌ | ✅ (avatars) | ✅ (avatars) |
| TTS | ✅ 15+ providers | ✅ | ❌ | ✅ (own) | ❌ |
| Image generation | ✅ | ✅ | ✅ | ❌ | ✅ |
| RAG/Data Bank | ✅ | ❌ | ✅ (pipeline) | ❌ | ❌ |
| Translation | ✅ | ✅ | ❌ | ✅ | ❌ |
| Web search | ✅ | ❌ | ❌ | ✅ | ❌ |
| Extension system | ✅ | ✅ (plugins) | ❌ | ❌ | ❌ |
| Scripting engine | ✅ (STscript) | ❌ | ❌ | ❌ | ❌ |
| Local-first | ✅ | ✅ | ✅ (self-host) | ❌ (cloud) | ❌ (cloud) |
| No content filter | ✅ | ✅ | ✅ | ❌ (filtered) | ✅ (no filter) |
| Structured personas | ❌ (flat text) | ❌ (flat text) | ❌ | ❌ | ❌ |
| Persona compression | ❌ | ❌ | ❌ | ❌ | ❌ |
| Local encryption | ❌ | ❌ | ❌ | ❌ | ❌ |
| Native desktop UI | ❌ (web) | ❌ (Tauri/web) | ❌ (web) | ❌ (web) | ❌ (web) |
| OS-native key storage | ❌ | ❌ | ❌ | ❌ | ❌ |

## MVP Recommendation

Prioritize:
1. **Multi-provider AI connection + streaming** — Without this, nothing else matters
2. **Character card CRUD + CharacterCardV2 spec** — Core data model, must work first
3. **Character library with search/tags** — Need to browse and find characters
4. **Conversation history + branching (swipes)** — Core interaction loop
5. **User personas with structured fields** — Key differentiator, inject into prompt
6. **Template variable resolution** (`{{user}}`, `{{char}}`, `{{persona.*}}`) — Cards from ecosystem break without this
7. **Lorebook with keyword activation** — Power users demand this. Table stakes for serious roleplay.
8. **Token counting + context budget** — Essential for managing limited context windows
9. **Generation parameter presets** — Expected by all users
10. **Markdown rendering** — AI output is unreadable without it

Defer: 
- **Image generation**: Not core. Adds huge integration surface. (Phase 3+)
- **TTS**: Not core. 15+ provider integrations is enormous scope. (Phase 3+)
- **Character sprites/expressions**: Requires sentiment classification. Complex GTK4 adaptation. (Phase 3+)
- **RAG/Data Bank**: Lorebook covers 80% of knowledge injection use cases. (Phase 4+)
- **Extension system**: Architectural concern. Ship solid core first. (v2+)
- **STscript/scripting**: Enormous scope. Slash commands suffice. (Never or v3+)
- **Group chat**: Complex multi-character prompt management. Do solo chat perfectly first. (Phase 2)

## Key Insight: The Desktop-App Differentiation

**Every major AI roleplay platform is web-based.** SillyTavern runs a Node.js server and renders in the browser. RisuAI is a Tauri app (Rust backend + web frontend). Agnai is a web app. Character.ai and SpicyChat are cloud web apps.

Revelry's GTK4 native desktop experience is genuinely unique in this space. This isn't just a technical choice — it's a product positioning:

- **Privacy users** who don't want to run a local web server
- **GNOME/Linux users** who want native HIG-compliant software
- **Performance users** who want lower latency than browser rendering
- **Power users** who want OS-level integration (keychain, file manager, notifications)

The feature strategy should lean into this: every feature that benefits from native desktop integration (keychain, file dialogs, drag-and-drop, system notifications, offline operation) should be prioritized. Every feature that's "web-native" (social sharing, cloud sync, marketplace) should be avoided.

## Sources

- SillyTavern Documentation: https://docs.sillytavern.app/ (HIGH confidence — official docs, verified via Context7)
- SillyTavern GitHub README: https://github.com/SillyTavern/SillyTavern (HIGH confidence — official repo)
- RisuAI GitHub: https://github.com/kwaroran/Risuai (HIGH confidence — official repo)
- Agnai GitHub: https://github.com/agnaistic/agnai (HIGH confidence — official repo)
- Character.ai: https://character.ai/ (MEDIUM confidence — 403 on direct fetch, feature knowledge from training data + community)
- SpicyChat: https://spicychat.ai/ (LOW confidence — minimal content retrieved, feature knowledge from training data)
- SillyTavern World Info docs: https://docs.sillytavern.app/usage/core-concepts/worldinfo/ (HIGH confidence)
- SillyTavern Personas docs: https://docs.sillytavern.app/usage/core-concepts/personas/ (HIGH confidence)
- SillyTavern Group Chats docs: https://docs.sillytavern.app/usage/core-concepts/groupchats/ (HIGH confidence)
- SillyTavern Character Design docs: https://docs.sillytavern.app/usage/core-concepts/characterdesign/ (HIGH confidence)
- SillyTavern Data Bank docs: https://docs.sillytavern.app/usage/core-concepts/data-bank/ (HIGH confidence)
- SillyTavern Macros docs: https://docs.sillytavern.app/usage/core-concepts/macros/ (HIGH confidence)
- SillyTavern TTS docs: https://docs.sillytavern.app/extensions/tts/ (HIGH confidence)
- SillyTavern Expression Images docs: https://docs.sillytavern.app/extensions/expression-images/ (HIGH confidence)
- PROJECT.md: Local project context (HIGH confidence — project's own document)
