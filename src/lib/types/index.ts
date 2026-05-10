/**
 * Type definitions mirroring the Revelry data model from the design spec.
 * These map directly to the SQLite tables in §12.1 of the spec.
 */

// ─────────────────────────────────────────────────────────────────────────────
// Character cards (CharacterCardV2)
// ─────────────────────────────────────────────────────────────────────────────

export interface CharacterCard {
	id: string;
	name: string;
	description: string;
	personality?: string;
	scenario?: string;
	first_mes?: string;
	mes_example?: string;
	creator_notes?: string;
	system_prompt?: string;
	post_history_instructions?: string;
	alternate_greetings?: string[];
	tags?: string[];
	creator?: string;
	character_version?: string;
	avatar_url?: string;
	extensions?: CharacterExtensions;
	created_at: string;
	updated_at: string;
}

export interface CharacterExtensions {
	default_persona_id?: string;
	rig_provider_override?: string;
	[key: string]: unknown;
}

// ─────────────────────────────────────────────────────────────────────────────
// Lorebook
// ─────────────────────────────────────────────────────────────────────────────

export type LorebookPosition = 'before_char' | 'after_char' | 'at_depth';
export type LorebookActivation = 'always' | 'character_only' | 'user_only';

export interface LorebookEntry {
	id: string;
	character_id: string;
	keys: string[];
	secondary_keys?: string[];
	content: string;
	position: LorebookPosition;
	depth?: number;
	activation_state: LorebookActivation;
	extensions?: Record<string, unknown>;
}

// ─────────────────────────────────────────────────────────────────────────────
// User personas
// ─────────────────────────────────────────────────────────────────────────────

export interface UserPersona {
	id: string;
	name: string;
	avatar_url?: string;
	appearance: string;
	personality: string;
	background?: string;
	role_description?: string;
	relationships?: string;
	speech_style?: string;
	notes?: string;
	tags?: string[];
	is_default?: boolean;
	extensions?: Record<string, unknown>;
	created_at: string;
	updated_at: string;
}

// ─────────────────────────────────────────────────────────────────────────────
// Conversations & messages
// ─────────────────────────────────────────────────────────────────────────────

export type MessageRole = 'user' | 'character' | 'system';

export interface Message {
	id: string;
	conversation_id: string;
	parent_id: string | null;
	branch_index: number;
	role: MessageRole;
	content: string;
	token_count?: number;
	metadata?: MessageMetadata;
	created_at: string;
	/** UI-only: true while tokens are still streaming in */
	is_streaming?: boolean;
}

export interface MessageMetadata {
	persona_id?: string;
	provider?: string;
	model?: string;
	stop_reason?: 'end' | 'cancelled' | 'length' | 'error';
	[key: string]: unknown;
}

export interface Conversation {
	id: string;
	character_id: string;
	user_persona_id?: string;
	title: string;
	created_at: string;
	updated_at: string;
	active_branch_tip?: string;
	/** Denormalized for the sidebar */
	last_message_preview?: string;
	message_count?: number;
}

// ─────────────────────────────────────────────────────────────────────────────
// Providers & generation
// ─────────────────────────────────────────────────────────────────────────────

export type ProviderType =
	| 'openai'
	| 'anthropic'
	| 'ollama'
	| 'gemini'
	| 'cohere'
	| 'mistral'
	| 'deepseek'
	| 'groq'
	| 'azure_openai'
	| 'openrouter'
	| 'together'
	| 'huggingface'
	| 'perplexity'
	| 'xai'
	| 'custom_openai_compatible';

export interface Provider {
	id: string;
	name: string;
	provider_type: ProviderType;
	base_url?: string;
	model: string;
	default_params?: GenerationParams;
	extensions?: Record<string, unknown>;
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

export interface PromptPreset {
	id: string;
	name: string;
	system_prompt?: string;
	temperature: number;
	top_p: number;
	max_tokens: number;
	frequency_penalty: number;
	presence_penalty: number;
	stop_sequences?: string[];
}

// ─────────────────────────────────────────────────────────────────────────────
// View routing
// ─────────────────────────────────────────────────────────────────────────────

export type PrimaryView = 'chat' | 'characters' | 'settings';
export type CharactersTab = 'characters' | 'personas';
