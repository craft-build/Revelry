import type {
  CharacterCard,
  Conversation,
  Message,
  UserPersona,
  Provider,
  GenerationPreset,
} from "$lib/types";
import {
  getCharacters,
  getPersonas,
  getConversations,
  getPresets,
  updateConversationPersona,
} from "$lib/data/api";

/**
 * Root application state. Mirrors the AppModel/ChatModel/CharacterModel/SettingsModel
 * split from §6.2 of the design spec, but unified into a single Svelte 5 rune-store
 * that components reactively subscribe to.
 *
 * Hydrated from the Rust backend over Tauri IPC using typed invoke() wrappers.
 */
class AppState {
  // ── Library ────────────────────────────────────────────────────────────
  characters = $state<CharacterCard[]>([]);
  personas = $state<UserPersona[]>([]);
  conversations = $state<Conversation[]>([]);
  providers = $state<Provider[]>([]);
  presets = $state<GenerationPreset[]>([]);

  // ── Loading / error state ──────────────────────────────────────────────
  is_loading = $state<boolean>(true);
  init_error = $state<string | null>(null);

  // ── Active conversation ────────────────────────────────────────────────
  active_conversation_id = $state<string | null>(null);
  messages = $state<Message[]>([]);
  is_generating = $state<boolean>(false);

  // ── UI state ───────────────────────────────────────────────────────────
  sidebar_open = $state<boolean>(true);
  character_search_query = $state<string>("");

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
    const persona_id = conv?.creator_persona_id;
    if (!persona_id) return undefined;
    return this.personas.find((p) => p.id === persona_id);
  }

  get filtered_characters(): CharacterCard[] {
    const q = this.character_search_query.trim().toLowerCase();
    if (!q) return this.characters;
    return this.characters.filter(
      (c) =>
        c.name.toLowerCase().includes(q) ||
        c.description.toLowerCase().includes(q) ||
        c.tags?.some((t) => t.toLowerCase().includes(q)),
    );
  }

  // ── Actions ────────────────────────────────────────────────────────────
  async hydrate() {
    this.is_loading = true;
    this.init_error = null;

    try {
      const results = await Promise.allSettled([
        getCharacters(),
        getPersonas(),
        getConversations(),
        getPresets(),
      ]);

      const [characters, personas, conversations, presets] = results;

      if (characters.status === "fulfilled") {
        this.characters = characters.value;
      } else {
        console.error("Failed to load characters:", characters.reason);
      }

      if (personas.status === "fulfilled") {
        this.personas = personas.value;
      } else {
        console.error("Failed to load personas:", personas.reason);
      }

      if (conversations.status === "fulfilled") {
        this.conversations = conversations.value;
      } else {
        console.error("Failed to load conversations:", conversations.reason);
      }

      if (presets.status === "fulfilled") {
        this.presets = presets.value;
      } else {
        console.error("Failed to load presets:", presets.reason);
      }

      // If any critical data failed, set error state
      const criticalFailures = results
        .slice(0, 3) // characters, personas, conversations are critical
        .filter((r) => r.status === "rejected");

      if (criticalFailures.length > 0) {
        const firstError = criticalFailures[0] as PromiseRejectedResult;
        this.init_error =
          typeof firstError.reason === "string"
            ? firstError.reason
            : (firstError.reason?.message ?? "Failed to load application data");
      }
    } catch (err) {
      this.init_error =
        err instanceof Error
          ? err.message
          : "Unknown error during initialization";
    } finally {
      this.is_loading = false;
    }
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
      m.id === id ? { ...m, content, is_streaming: !done } : m,
    );
    if (done) this.is_generating = false;
  }

  toggleSidebar() {
    this.sidebar_open = !this.sidebar_open;
  }

  async switchPersona(persona_id: string) {
    if (!this.active_conversation_id) return;
    // Optimistically update local state
    this.conversations = this.conversations.map((c) =>
      c.id === this.active_conversation_id
        ? { ...c, creator_persona_id: persona_id }
        : c,
    );
    // Persist to backend
    try {
      await updateConversationPersona(this.active_conversation_id, persona_id);
    } catch (err) {
      console.error("Failed to persist persona switch:", err);
      // Revert on failure
      this.conversations = this.conversations.map((c) =>
        c.id === this.active_conversation_id
          ? { ...c, creator_persona_id: undefined }
          : c,
      );
    }
  }
}

export const app = new AppState();
