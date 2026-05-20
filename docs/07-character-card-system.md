<!-- Revelry Design Specification v2.0 — Section 7 -->
<!-- Last updated: 2026-05-20 -->

# Character Card System

The character card is the atomic unit of Revelry's data model, encoding everything the AI needs to portray a character consistently. Revelry adopts the CharacterCardV2 specification established by the SillyTavern community, ensuring maximum interoperability with the existing ecosystem of character cards, while extending it with additional fields for user persona integration and the rig provider configuration.

## 7.1 Card Format Specification

Character cards are stored as PNG images with embedded JSON metadata (following the tEXt chunk convention) for visual portability, or as standalone JSON files for programmatic access. The V2 specification defines a structured data model with nested objects for different aspects of a character's definition. Each card contains the following primary fields:

| **Field** | **Type** | **Required** | **Description** |
|---|---|---|---|
| name | string | Yes | Character display name, used in chat headers and mentions |
| description | string | Yes | Core personality and background, always included in context |
| personality | string | No | Summary of personality traits, appended to description |
| scenario | string | No | Starting scenario / setting for new conversations |
| first_mes | string | No | Character's opening message when starting a new chat |
| mes_example | string | No | Sample dialogue demonstrating character's speech patterns |
| creator_notes | string | No | Notes from the card creator for users |
| system_prompt | string | No | Custom system prompt override for this character |
| post_history_instructions | string | No | Instructions injected after conversation history |
| alternate_greetings | string[] | No | Additional first messages beyond first_mes |
| tags | string[] | No | Categorization tags for search and filtering |
| creator | string | No | Name of the card creator |
| character_version | string | No | Version string for the card |
| extensions | object | No | Namespace for app-specific extensions (personas, rig config) |

## 7.2 Lorebook System

The lorebook (also known as World Info) is a critical extension that enables rich world-building within the character card framework. A lorebook is a collection of entries, each containing a set of keywords, activation conditions, and content text. When the keywords appear in the recent conversation, the entry's content is automatically injected into the AI's context, providing relevant background information without cluttering the permanent character description.

Each lorebook entry specifies: key terms (comma-separated keywords that trigger activation), secondary terms (additional keywords that must co-occur with primary terms), content (the text injected into context when activated), insertion position (before the character description, after it, or at a specific depth in the conversation), activation state (always active, only when character is speaking, or only when user mentions the keyword), and recursion depth (whether activated entries can trigger other entries, up to a configurable depth limit).

Revelry extends the lorebook system with persona-aware activation, where entries can be configured to activate based on the active user persona's data fields in addition to conversation text. This enables characters to have special knowledge or reactions that only surface when interacting with specific types of user personas, creating richer first-impression interactions.

## 7.3 Card Import/Export

Revelry supports importing character cards from multiple sources. PNG cards with embedded V2 metadata can be loaded through GtkFileChooserNative or drag-and-drop onto the application window. JSON card files are also supported, with automatic detection of V1 and V2 formats and migration to V2 when needed. Cards from SillyTavern, CharacterAI (via community export tools), and other compatible platforms are recognized and their data mapped to Revelry's internal schema. Export functionality allows users to save cards as PNG (with the character's avatar image and embedded metadata) or JSON formats.

## 7.4 Character Creation Wizard

Creating a new character in Revelry follows a guided, multi-step AdwPreferencesWindow that breaks the card definition into digestible sections. Step 1 covers identity (name, avatar upload through GtkFileChooserNative, and a short tagline). Step 2 focuses on personality (description, personality traits, and behavioral patterns). Step 3 addresses the scenario (starting situation, world context, and first message). Step 4 handles advanced settings (sample dialogues, system prompt overrides, and lorebook initialization). Each step provides contextual guidance text explaining what information to include and how it affects the AI's behavior.
