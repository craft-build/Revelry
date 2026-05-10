import type {
	CharacterCard,
	Conversation,
	Message,
	UserPersona,
	Provider,
	PromptPreset
} from '$lib/types';

/**
 * Root application state. Mirrors the AppModel/ChatModel/CharacterModel/SettingsModel
 * split from §6.2 of the design spec, but unified into a single Svelte 5 rune-store
 * that components reactively subscribe to.
 *
 * In the production app this will be hydrated from the Rust backend over Tauri/IPC
 * (or an HTTP API if deployed as a web app). For the scaffold we hydrate from
 * mock data in $lib/data/mock.ts.
 */
class AppState {
	// ── Library ────────────────────────────────────────────────────────────
	characters = $state<CharacterCard[]>([]);
	personas = $state<UserPersona[]>([]);
	conversations = $state<Conversation[]>([]);
	providers = $state<Provider[]>([]);
	prompt_presets = $state<PromptPreset[]>([]);

	// ── Active conversation ────────────────────────────────────────────────
	active_conversation_id = $state<string | null>(null);
	messages = $state<Message[]>([]);
	is_generating = $state<boolean>(false);

	// ── UI state ───────────────────────────────────────────────────────────
	sidebar_open = $state<boolean>(true);
	character_search_query = $state<string>('');

	// ── Derived ────────────────────────────────────────────────────────────
	get active_conversation(): Conversation | undefined {
		return this.conversations.find((c) => c.id === this.active_conversation_id);
	}

	get active_character(): CharacterCard | undefined {
		const conv = this.active_conversation;
		if (!conv) return undefined;
		return this.characters.find((c) => c.id === conv.character_id);
	}

	get active_persona(): UserPersona | undefined {
		const conv = this.active_conversation;
		const persona_id = conv?.user_persona_id ?? this.default_persona?.id;
		if (!persona_id) return undefined;
		return this.personas.find((p) => p.id === persona_id);
	}

	get default_persona(): UserPersona | undefined {
		return this.personas.find((p) => p.is_default);
	}

	get filtered_characters(): CharacterCard[] {
		const q = this.character_search_query.trim().toLowerCase();
		if (!q) return this.characters;
		return this.characters.filter(
			(c) =>
				c.name.toLowerCase().includes(q) ||
				c.description.toLowerCase().includes(q) ||
				c.tags?.some((t) => t.toLowerCase().includes(q))
		);
	}

	// ── Actions ────────────────────────────────────────────────────────────
	hydrate(data: {
		characters: CharacterCard[];
		personas: UserPersona[];
		conversations: Conversation[];
		providers: Provider[];
		prompt_presets: PromptPreset[];
	}) {
		this.characters = data.characters;
		this.personas = data.personas;
		this.conversations = data.conversations;
		this.providers = data.providers;
		this.prompt_presets = data.prompt_presets;
	}

	openConversation(id: string, messages: Message[]) {
		this.active_conversation_id = id;
		this.messages = messages;
	}

	closeConversation() {
		this.active_conversation_id = null;
		this.messages = [];
	}

	appendMessage(message: Message) {
		this.messages = [...this.messages, message];
	}

	updateStreamingMessage(id: string, content: string, done = false) {
		this.messages = this.messages.map((m) =>
			m.id === id ? { ...m, content, is_streaming: !done } : m
		);
		if (done) this.is_generating = false;
	}

	toggleSidebar() {
		this.sidebar_open = !this.sidebar_open;
	}

	switchPersona(persona_id: string) {
		if (!this.active_conversation_id) return;
		this.conversations = this.conversations.map((c) =>
			c.id === this.active_conversation_id ? { ...c, user_persona_id: persona_id } : c
		);
	}
}

export const app = new AppState();
