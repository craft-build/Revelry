/**
 * Type definitions mirroring the Revelry data model.
 * These map to the Rust domain types serialized via serde in the Tauri backend.
 */

// ─────────────────────────────────────────────────────────────────────────────
// Character cards (CharacterCardV2)
// ─────────────────────────────────────────────────────────────────────────────

export interface CharacterCard {
  id: string;
  name: string;
  description: string;
  personality: string;
  scenario: string;
  first_mes: string;
  mes_example: string;
  creator_notes: string;
  system_prompt: string;
  post_history_instructions: string;
  alternate_greetings: string[];
  tags: string[];
  creator: string;
  character_version: string;
  avatar_path?: string;
  extensions?: Record<string, unknown> | null;
  /** Per-character default persona ID for persona assignment hierarchy. */
  default_persona_id?: string;
  /** Per-character default preset ID for generation parameter hierarchy per D-03. */
  default_preset_id?: string;
  /** Lorebook / world info (JSON). */
  character_book?: Record<string, unknown> | null;
  lorebook_scan_depth: number;
  lorebook_recursive_depth: number;
  created_at: string;
  updated_at: string;
}

// ─────────────────────────────────────────────────────────────────────────────
// Lorebook
// ─────────────────────────────────────────────────────────────────────────────

export type LorebookPosition = "before_char" | "after_char" | "at_depth";
export type LorebookActivation = "always" | "character_only" | "user_only";

export interface Lorebook {
  id: string;
  name: string;
  description: string;
  created_at: string;
  updated_at: string;
}

export interface LorebookEntry {
  id: string;
  lorebook_id: string;
  keys: string[];
  secondary_keys?: string[];
  content: string;
  position: LorebookPosition;
  depth: number;
  activation_state: LorebookActivation;
  extensions?: Record<string, unknown> | null;
  created_at: string;
  updated_at: string;
}

export interface KeywordTestResult {
  entry_id: string;
  matched_keys: string[];
}

// ─────────────────────────────────────────────────────────────────────────────
// User personas
// ─────────────────────────────────────────────────────────────────────────────

export interface UserPersona {
  id: string;
  name: string;
  avatar_path?: string;
  appearance: string;
  personality: string;
  background: string;
  role_description: string;
  relationships: string;
  speech_style: string;
  tags: string[];
  created_at: string;
  updated_at: string;
}

// ─────────────────────────────────────────────────────────────────────────────
// Conversations & messages
// ─────────────────────────────────────────────────────────────────────────────

export type MessageRole = "user" | "assistant" | "system";

export interface Message {
  id: string;
  conversation_id: string;
  role: MessageRole;
  content: string;
  sort_order: number;
  timestamp: string;
  /** UI-only: true while tokens are still streaming in */
  is_streaming?: boolean;
  /** UI-only: token count estimate for display */
  token_count?: number;
  /** UI-only: generation metadata */
  metadata?: MessageMetadata;
}

export interface MessageMetadata {
  persona_id?: string;
  provider?: string;
  model?: string;
  stop_reason?: "end" | "cancelled" | "length" | "error";
  [key: string]: unknown;
}

export interface Conversation {
  id: string;
  character_id: string;
  /** Nullable — no providers in Phase 2. */
  provider_id?: string;
  /** Specific model within provider. */
  model_name?: string;
  creator_persona_id?: string;
  /** Per-conversation preset override per D-03. */
  generation_preset_id?: string;
  /** Display title for the conversation. */
  title: string;
  /** Truncated preview of the last message for list display. */
  last_message_preview: string;
  created_at: string;
  updated_at: string;
}

// ─────────────────────────────────────────────────────────────────────────────
// Providers & generation
// ─────────────────────────────────────────────────────────────────────────────

export type ProviderType =
  | "openai"
  | "anthropic"
  | "ollama"
  | "gemini"
  | "cohere"
  | "mistral"
  | "deepseek"
  | "groq"
  | "azure_openai"
  | "openrouter"
  | "together"
  | "huggingface"
  | "perplexity"
  | "xai"
  | "custom_openai_compatible";

export interface ProviderModelEntry {
  name: string;
  context_window_size?: number | null;
  enabled: boolean;
}

export interface Provider {
  id: string;
  name: string;
  provider_type: ProviderType;
  base_url?: string;
  model_name: string;
  models: ProviderModelEntry[];
  context_window_size?: number;
  default_params?: GenerationParams;
  has_api_key: boolean;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface GenerationParams {
  temperature?: number;
  top_p?: number;
  max_tokens?: number;
  frequency_penalty?: number;
  presence_penalty?: number;
  stop_sequences?: string[];
  context_length?: number;
}

export interface GenerationPreset {
  id: string;
  name: string;
  is_builtin: boolean;
  temperature: number;
  top_p: number;
  max_tokens: number;
  frequency_penalty: number;
  presence_penalty: number;
  stop_sequences: string[];
  created_at: string;
  updated_at: string;
}

/** Backward-compat alias — prefer GenerationPreset. */
export type { GenerationPreset as PromptPreset };

// ─────────────────────────────────────────────────────────────────────────────
// View routing
// ─────────────────────────────────────────────────────────────────────────────

export type PrimaryView = "chat" | "characters" | "settings";
export type CharactersTab = "characters" | "personas" | "lorebooks";
