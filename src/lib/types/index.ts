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
/** How an entry decides to activate (Phase 11). */
export type LorebookActivationMode = "keyword" | "semantic" | "hybrid";

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
  /** Activation mode — keyword (default), semantic, or hybrid. */
  activation_mode?: LorebookActivationMode;
  /** Cosine threshold for semantic/hybrid activation (default 0.6). */
  semantic_threshold?: number;
  /** Per-entry soft token limit (nullable). */
  token_budget?: number | null;
  /** Display/activation ordering. */
  sort_order: number;
  /** Activation toggle. */
  enabled: boolean;
  extensions?: Record<string, unknown> | null;
  created_at: string;
  updated_at: string;
}

export interface KeywordTestResult {
  entry_id: string;
  matched_keys: string[];
}

/** Per-entry semantic activation preview from test_semantic_activation. */
export interface SemanticTestResult {
  entryId: string;
  keys: string[];
  activationMode: string;
  similarity?: number | null;
  threshold: number;
  activated: boolean;
  reason?: string | null;
}

export interface LorebookActivationDetail {
  lorebook_name: string;
  entry_name: string;
  content_snippet: string;
  token_cost: number;
  matched_keys?: string[];
  /** "keyword" | "semantic" | "always" — how the entry activated. */
  activation_reason?: string | null;
  /** Cosine similarity when evaluated semantically. */
  similarity?: number | null;
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
  /** Branch this message belongs to. null = Main/root path. */
  branch_id?: string | null;
  /** UI-only: true while tokens are still streaming in */
  is_streaming?: boolean;
  /** UI-only: token count estimate for display */
  token_count?: number;
  /** UI-only: generation metadata */
  metadata?: MessageMetadata;
  /** UI-only: steering instruction extracted from this user message (for rendering). */
  steering?: string;
}

export interface MessageMetadata {
  persona_id?: string;
  provider?: string;
  model?: string;
  stop_reason?: "end" | "cancelled" | "length" | "error";
  error_type?: string;
  [key: string]: unknown;
}

/** Discriminated union matching Rust StreamChunk serde output.
 * Tagged with `event` field, payload in `data` field. */
export type StreamChunk =
  | { event: "token"; data: { content: string } }
  | {
      event: "complete";
      data: {
        messageId: string;
        content: string;
        stopReason: string;
        tokensUsed: number;
        tokensLimit: number;
        truncatedCount: number;
        lorebookDetails?: LorebookActivationDetail[];
        cancelled?: boolean;
        /** Memory-layer outcome for this prompt. */
        memory?: MemoryInfo;
      };
    }
  | { event: "error"; data: { error: string; errorType: string } };

/** Memory-layer outcome on the Complete chunk (summary + recall). */
export interface MemoryInfo {
  summaryIncluded: boolean;
  recalledCount: number;
  externalCount: number;
}

/** Context budget info returned from backend after each generation. */
export interface ContextBudget {
  tokensUsed: number;
  tokensLimit: number;
  truncatedCount: number;
  lorebookDetails?: LorebookActivationDetail[];
  memory?: MemoryInfo;
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
  /** Per-conversation max output token override. Takes precedence over preset and provider defaults. */
  override_max_tokens?: number | null;
  /** Display title for the conversation. */
  title: string;
  /** Truncated preview of the last message for list display. */
  last_message_preview: string;
  /** Rolling story summary (memory layer). Empty until generated. */
  summary?: string;
  /** Highest message sort_order covered by the summary. */
  summary_upto?: number;
  created_at: string;
  updated_at: string;
}

/** A branch in a conversation, created when a user forks from a message. */
export interface MessageBranch {
  id: string;
  conversation_id: string;
  parent_message_id: string;
  label: string;
  created_at: string;
}

/** A search result from FTS5 message search, conversation title search, or
 *  semantic (embedding) message search. */
export interface SearchResult {
  conversation_id: string;
  conversation_title: string;
  character_id: string;
  match_type: "title" | "message" | "semantic";
  message_snippet?: string;
  message_id?: string;
  branch_id?: string;
  rank: number;
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
// Regex scripts
// ─────────────────────────────────────────────────────────────────────────────

export interface RegexScript {
  id: string;
  character_id: string;
  name: string;
  find_pattern: string;
  replace_pattern: string;
  flags: string;
  sort_order: number;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface RegexTestResult {
  output: string;
  match_count: number;
}

// ─────────────────────────────────────────────────────────────────────────────
// Character generation
// ─────────────────────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────────────────────
// Embeddings & semantic retrieval
// ─────────────────────────────────────────────────────────────────────────────

/** Embedding source selection. Mirrors Rust EmbeddingConfig (serde-tagged). */
export type EmbeddingConfig =
  | { source: "none" }
  | { source: "remote"; providerId: string; model: string }
  | { source: "local"; model: string };

export interface EmbeddingNamespaceCount {
  namespace: string;
  count: number;
}

export interface EmbeddingStatus {
  configured: boolean;
  source: "none" | "remote" | "local";
  modelKey: string;
  counts: EmbeddingNamespaceCount[];
  staleCount: number;
}

export interface EmbeddingTestResult {
  dims: number;
  latencyMs: number;
}

/** Local fastembed model choice surfaced in Settings. */
export interface LocalEmbeddingModel {
  key: string;
  label: string;
}

export const LOCAL_EMBEDDING_MODELS: LocalEmbeddingModel[] = [
  {
    key: "nomic-embed-text-v1.5",
    label: "Nomic embed v1.5 · 768 dims · ~100 MB",
  },
  { key: "bge-small-en-v1.5", label: "BGE small EN v1.5 · 384 dims · ~35 MB" },
];

// ─────────────────────────────────────────────────────────────────────────────
// Character generation
// ─────────────────────────────────────────────────────────────────────────────

/** Streaming chunk for the character generation command. */
export type GenerateCharacterChunk =
  | { event: "token"; data: { content: string } }
  | { event: "complete"; data: { character: GeneratedCharacterData } }
  | { event: "error"; data: { error: string; errorType: string } };

/** Partial character data returned by the generator. Creative fields only —
 *  id, created_at, updated_at are assigned on save. */
export interface GeneratedCharacterData {
  name: string;
  description: string;
  personality: string;
  scenario: string;
  first_mes: string;
  mes_example: string;
  system_prompt: string;
  post_history_instructions: string;
  creator_notes: string;
  tags: string[];
  alternate_greetings: string[];
}
