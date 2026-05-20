import type {
  CharacterCard,
  Conversation,
  Message,
  MessageBranch,
  SearchResult,
  UserPersona,
  Provider,
  GenerationPreset,
  Lorebook,
  LorebookActivationDetail,
  RegexScript,
  RegexTestResult,
  StreamChunk,
  ContextBudget,
} from "$lib/types";
import { Channel } from "@tauri-apps/api/core";
import {
  getCharacters as apiGetCharacters,
  saveCharacter as apiSaveCharacter,
  deleteCharacter as apiDeleteCharacter,
  getPersonas as apiGetPersonas,
  savePersona as apiSavePersona,
  deletePersona as apiDeletePersona,
  getConversations as apiGetConversations,
  startConversation as apiStartConversation,
  deleteConversation as apiDeleteConversation,
  getPresets as apiGetPresets,
  updateConversationPersona as apiUpdateConversationPersona,
  updateConversationProvider as apiUpdateConversationProvider,
  getLorebooks as apiGetLorebooks,
  saveLorebook as apiSaveLorebook,
  deleteLorebook as apiDeleteLorebook,
  uploadAvatar as apiUploadAvatar,
  deleteAvatar as apiDeleteAvatar,
  getProviders as apiGetProviders,
  saveProvider as apiSaveProvider,
  deleteProvider as apiDeleteProvider,
  saveApiKey as apiSaveApiKey,
  deleteApiKey as apiDeleteApiKey,
  checkKeyringAvailable as apiCheckKeyringAvailable,
  assignLorebookToCharacter as apiAssignLorebookToCharacter,
  removeLorebookFromCharacter as apiRemoveLorebookFromCharacter,
  getCharacterLorebooks as apiGetCharacterLorebooks,
  getConversationLorebooks as apiGetConversationLorebooks,
  assignLorebookToConversation as apiAssignLorebookToConversation,
  removeLorebookFromConversation as apiRemoveLorebookFromConversation,
  getRegexScripts as apiGetRegexScripts,
  saveRegexScript as apiSaveRegexScript,
  deleteRegexScript as apiDeleteRegexScript,
  applySingleRegex as apiApplySingleRegex,
  startChat as apiStartChat,
  stopGeneration as apiStopGeneration,
  regenerate as apiRegenerate,
  editAndResend as apiEditAndResend,
  deleteMessage as apiDeleteMessage,
  getMessages as apiGetMessages,
  forkConversation as apiForkConversation,
  getBranchesForConversation as apiGetBranchesForConversation,
  deleteBranch as apiDeleteBranch,
  searchMessages as apiSearchMessages,
  saveMessage as apiSaveMessage,
  importCard as apiImportCard,
  exportCard as apiExportCard,
} from "$lib/data/api";
import { open, save } from "@tauri-apps/plugin-dialog";

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
  lorebooks = $state<Lorebook[]>([]);
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

  // ── Context budget ─────────────────────────────────────────────────────
  context_budget = $state<ContextBudget | null>(null);

  // ── UI state ───────────────────────────────────────────────────────────
  sidebar_open = $state<boolean>(true);
  character_search_query = $state<string>("");
  keyring_available = $state<boolean>(true);
  focusProviderSelector = $state<boolean>(false);

  // ── Branch state ───────────────────────────────────────────────────────
  active_branch_id = $state<string | null>(null);
  branches = $state<MessageBranch[]>([]);
  search_results = $state<SearchResult[]>([]);
  is_searching = $state<boolean>(false);

  // ── Regex & lorebook state ───────────────────────────────────────────────
  regex_scripts = $state<Map<string, RegexScript[]>>(new Map());
  conversation_lorebooks = $state<Lorebook[]>([]);
  character_lorebooks = $state<Lorebook[]>([]);
  lorebook_detail_open = $state<boolean>(false);
  lorebook_details = $state<LorebookActivationDetail[]>([]);

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

  get active_provider(): Provider | undefined {
    const conv = this.active_conversation;
    if (!conv?.provider_id) return undefined;
    return this.providers.find((p) => p.id === conv.provider_id);
  }

  get active_branch(): MessageBranch | undefined {
    if (!this.active_branch_id) return undefined;
    return this.branches.find((b) => b.id === this.active_branch_id);
  }

  /** Effective context window size for the active conversation.
   *  1. Use backend-reported budget limit if available
   *  2. Fall back to the model entry's context_window_size
   *  3. Fall back to the provider-level context_window_size
   *  4. Default to 8192 */
  get effective_context_limit(): number {
    if (this.context_budget) return this.context_budget.tokensLimit;
    const provider = this.active_provider;
    if (!provider) return 8192;
    const conv = this.active_conversation;
    const modelName = conv?.model_name ?? provider.model_name;
    const modelEntry = provider.models.find((m) => m.name === modelName);
    if (modelEntry?.context_window_size) return modelEntry.context_window_size;
    if (provider.context_window_size) return provider.context_window_size;
    return 8192;
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

  get filtered_lorebooks(): Lorebook[] {
    const q = this.character_search_query.trim().toLowerCase();
    if (!q) return this.lorebooks;
    return this.lorebooks.filter(
      (l) =>
        l.name.toLowerCase().includes(q) ||
        l.description.toLowerCase().includes(q),
    );
  }

  get filtered_personas(): UserPersona[] {
    const q = this.character_search_query.trim().toLowerCase();
    if (!q) return this.personas;
    return this.personas.filter(
      (p) =>
        p.name.toLowerCase().includes(q) ||
        p.personality.toLowerCase().includes(q) ||
        p.tags?.some((t) => t.toLowerCase().includes(q)),
    );
  }

  // ── Actions ────────────────────────────────────────────────────────────
  async hydrate() {
    this.is_loading = true;
    this.init_error = null;

    try {
      const results = await Promise.allSettled([
        apiGetCharacters(),
        apiGetPersonas(),
        apiGetLorebooks(),
        apiGetConversations(),
        apiGetPresets(),
        apiGetProviders(),
      ]);

      const [
        characters,
        personas,
        lorebooks,
        conversations,
        presets,
        providers,
      ] = results;

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

      if (lorebooks.status === "fulfilled") {
        this.lorebooks = lorebooks.value;
      } else {
        console.error("Failed to load lorebooks:", lorebooks.reason);
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

      if (providers.status === "fulfilled") {
        this.providers = providers.value;
      } else {
        console.error("Failed to load providers:", providers.reason);
      }

      // If any critical data failed, set error state
      const criticalFailures = results
        .slice(0, 4) // characters, personas, lorebooks, conversations are critical
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

  openConversation(id: string, messages: Message[], branchId?: string | null) {
    this.active_conversation_id = id;
    this.messages = messages;
    this.active_branch_id = branchId ?? null;
    // Load branches for this conversation
    this.loadBranchesForConversation(id);
    this.loadConversationLorebooks(id);
    // Load character lorebooks and regex scripts when opening conversation
    if (this.active_character) {
      this.loadCharacterLorebooks(this.active_character.id);
      this.loadRegexScripts(this.active_character.id);
    }
  }

  closeConversation() {
    this.active_conversation_id = null;
    this.messages = [];
    this.context_budget = null;
    this.active_branch_id = null;
    this.branches = [];
    this.conversation_lorebooks = [];
    this.character_lorebooks = [];
    this.lorebook_details = [];
    this.lorebook_detail_open = false;
  }

  appendMessage(message: Message) {
    this.messages = [...this.messages, message];
  }

  async deleteMessage(id: string): Promise<void> {
    const prev = this.messages;
    this.messages = this.messages.filter((m) => m.id !== id);
    try {
      await apiDeleteMessage(id);
    } catch (err) {
      console.error("Failed to delete message:", err);
      this.messages = prev;
      throw err;
    }
  }

  updateStreamingMessage(id: string, content: string, done = false) {
    this.messages = this.messages.map((m) =>
      m.id === id ? { ...m, content, is_streaming: !done } : m,
    );
    if (done) this.is_generating = false;
  }

  /** Update a conversation's last_message_preview and updated_at in the sidebar list. */
  updateConversationPreview(conversationId: string, content: string) {
    const preview = content.slice(0, 100);
    const now = new Date().toISOString();
    this.conversations = this.conversations.map((c) =>
      c.id === conversationId
        ? { ...c, last_message_preview: preview, updated_at: now }
        : c,
    );
  }

  /** Update context budget from backend response. */
  updateContextBudget(budget: ContextBudget) {
    this.context_budget = budget;
  }

  toggleSidebar() {
    this.sidebar_open = !this.sidebar_open;
  }

  async switchPersona(persona_id: string) {
    if (!this.active_conversation_id) return;
    // Save previous value for rollback
    const prevPersonaId = this.active_conversation?.creator_persona_id;
    // Optimistically update local state
    this.conversations = this.conversations.map((c) =>
      c.id === this.active_conversation_id
        ? { ...c, creator_persona_id: persona_id }
        : c,
    );
    // Persist to backend
    try {
      await apiUpdateConversationPersona(
        this.active_conversation_id,
        persona_id,
      );
    } catch (err) {
      console.error("Failed to persist persona switch:", err);
      // Revert to previous value on failure
      this.conversations = this.conversations.map((c) =>
        c.id === this.active_conversation_id
          ? { ...c, creator_persona_id: prevPersonaId }
          : c,
      );
    }
  }

  // ── Conversation actions ────────────────────────────────────────────────

  /** Load all branches for a conversation. */
  async loadBranchesForConversation(conversationId: string): Promise<void> {
    try {
      this.branches = await apiGetBranchesForConversation(conversationId);
    } catch (err) {
      console.error("Failed to load branches:", err);
      this.branches = [];
    }
  }

  /** Switch to a different branch. Reloads messages for that branch. */
  async switchBranch(branchId: string | null): Promise<void> {
    if (!this.active_conversation_id) return;
    this.active_branch_id = branchId;
    try {
      const messages = await apiGetMessages(
        this.active_conversation_id,
        branchId,
      );
      this.messages = messages;
    } catch (err) {
      console.error("Failed to switch branch:", err);
    }
  }

  /** Fork from a specific message, creating a new branch. */
  async forkFromMessage(
    messageId: string,
    isAiMessage: boolean,
  ): Promise<void> {
    if (!this.active_conversation_id) return;
    try {
      const branch = await apiForkConversation(
        this.active_conversation_id,
        messageId,
      );
      this.branches = [...this.branches, branch];
      this.active_branch_id = branch.id;
      // Load messages for the new branch
      const messages = await apiGetMessages(
        this.active_conversation_id,
        branch.id,
      );
      this.messages = messages;

      // If forking from an AI message, auto-regenerate on the new branch
      if (isAiMessage) {
        await this.regenerate();
      }
    } catch (err) {
      console.error("Failed to create branch:", err);
      throw err;
    }
  }

  /** Delete a branch. If it's the active branch, switch to Main. */
  async deleteBranchAction(branchId: string): Promise<void> {
    try {
      await apiDeleteBranch(branchId);
      this.branches = this.branches.filter((b) => b.id !== branchId);
      if (this.active_branch_id === branchId) {
        this.active_branch_id = null;
        // Reload root messages
        if (this.active_conversation_id) {
          const messages = await apiGetMessages(
            this.active_conversation_id,
            null,
          );
          this.messages = messages;
        }
      }
    } catch (err) {
      console.error("Failed to delete branch:", err);
      throw err;
    }
  }

  /** Cycle to the previous or next branch at a given fork point.
   *  The ordered list is [Main (null), branch1, branch2, ...].
   *  Wraps around at both ends. */
  async cycleBranchAtForkPoint(
    parentMessageId: string,
    direction: "prev" | "next",
  ): Promise<void> {
    const siblingBranches = this.branches.filter(
      (b) => b.parent_message_id === parentMessageId,
    );
    if (siblingBranches.length === 0) return;

    // Build ordered list of branch IDs: null = Main, then each branch
    const ordered: (string | null)[] = [
      null,
      ...siblingBranches.map((b) => b.id),
    ];
    const currentIdx = ordered.indexOf(this.active_branch_id);
    if (currentIdx === -1) return;

    const len = ordered.length;
    const nextIdx =
      direction === "next"
        ? (currentIdx + 1) % len
        : (currentIdx - 1 + len) % len;

    await this.switchBranch(ordered[nextIdx]);
  }

  /** Search across all conversations and messages via backend FTS5. */
  async searchMessagesAction(query: string): Promise<void> {
    if (!query.trim()) {
      this.search_results = [];
      return;
    }
    this.is_searching = true;
    try {
      this.search_results = await apiSearchMessages(query.trim());
    } catch (err) {
      console.error("Search failed:", err);
      this.search_results = [];
    } finally {
      this.is_searching = false;
    }
  }

  /** Navigate to a search result's conversation and optionally switch branch. */
  async navigateToSearchResult(result: SearchResult): Promise<void> {
    const messages = await apiGetMessages(
      result.conversation_id,
      result.branch_id ?? null,
    );
    this.openConversation(
      result.conversation_id,
      messages,
      result.branch_id ?? null,
    );
  }

  async startConversation(characterId: string): Promise<Conversation> {
    const conversation = await apiStartConversation(characterId);
    this.conversations = [conversation, ...this.conversations];
    return conversation;
  }

  async deleteConversation(id: string): Promise<void> {
    try {
      await apiDeleteConversation(id);
      this.conversations = this.conversations.filter((c) => c.id !== id);
      if (this.active_conversation_id === id) {
        this.closeConversation();
      }
    } catch (err) {
      console.error("Failed to delete conversation:", err);
      throw err;
    }
  }

  // ── Character CRUD ──────────────────────────────────────────────────────

  async createCharacter(
    data: Omit<CharacterCard, "id" | "created_at" | "updated_at"> & {
      id?: string;
    },
  ): Promise<CharacterCard> {
    const character: CharacterCard = {
      id: data.id ?? crypto.randomUUID(),
      name: data.name,
      description: data.description ?? "",
      personality: data.personality ?? "",
      scenario: data.scenario ?? "",
      first_mes: data.first_mes ?? "",
      mes_example: data.mes_example ?? "",
      creator_notes: data.creator_notes ?? "",
      system_prompt: data.system_prompt ?? "",
      post_history_instructions: data.post_history_instructions ?? "",
      alternate_greetings: data.alternate_greetings ?? [],
      tags: data.tags ?? [],
      creator: data.creator ?? "",
      character_version: data.character_version ?? "2.0",
      avatar_path: data.avatar_path,
      extensions: data.extensions,
      character_book: data.character_book,
      default_persona_id: data.default_persona_id,
      default_preset_id: data.default_preset_id,
      lorebook_scan_depth: data.lorebook_scan_depth ?? 2,
      lorebook_recursive_depth: data.lorebook_recursive_depth ?? 2,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    try {
      const saved = await apiSaveCharacter(character);
      this.characters = [saved, ...this.characters];
      return saved;
    } catch (err) {
      console.error("Failed to create character:", err);
      throw err;
    }
  }

  async updateCharacter(character: CharacterCard): Promise<CharacterCard> {
    const prev = this.characters.find((c) => c.id === character.id);
    try {
      const saved = await apiSaveCharacter({
        ...character,
        updated_at: new Date().toISOString(),
      });
      this.characters = this.characters.map((c) =>
        c.id === saved.id ? saved : c,
      );
      return saved;
    } catch (err) {
      console.error("Failed to update character:", err);
      // Revert on failure
      if (prev) {
        this.characters = this.characters.map((c) =>
          c.id === prev.id ? prev : c,
        );
      }
      throw err;
    }
  }

  async deleteCharacter(id: string): Promise<void> {
    const character = this.characters.find((c) => c.id === id);
    try {
      if (character?.avatar_path) {
        try {
          await apiDeleteAvatar(character.avatar_path);
        } catch (err) {
          console.warn("Failed to delete avatar file:", err);
        }
      }
      await apiDeleteCharacter(id);
      this.characters = this.characters.filter((c) => c.id !== id);
      if (this.active_character?.id === id) {
        this.closeConversation();
      }
    } catch (err) {
      console.error("Failed to delete character:", err);
      throw err;
    }
  }

  // ── Persona CRUD ────────────────────────────────────────────────────────

  async createPersona(
    data: Omit<UserPersona, "id" | "created_at" | "updated_at"> & {
      id?: string;
    },
  ): Promise<UserPersona> {
    const persona: UserPersona = {
      id: data.id ?? crypto.randomUUID(),
      name: data.name,
      avatar_path: data.avatar_path,
      appearance: data.appearance ?? "",
      personality: data.personality ?? "",
      background: data.background ?? "",
      role_description: data.role_description ?? "",
      relationships: data.relationships ?? "",
      speech_style: data.speech_style ?? "",
      tags: data.tags ?? [],
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    try {
      const saved = await apiSavePersona(persona);
      this.personas = [saved, ...this.personas];
      return saved;
    } catch (err) {
      console.error("Failed to create persona:", err);
      throw err;
    }
  }

  async updatePersona(persona: UserPersona): Promise<UserPersona> {
    const prev = this.personas.find((p) => p.id === persona.id);
    try {
      const saved = await apiSavePersona({
        ...persona,
        updated_at: new Date().toISOString(),
      });
      this.personas = this.personas.map((p) => (p.id === saved.id ? saved : p));
      return saved;
    } catch (err) {
      console.error("Failed to update persona:", err);
      if (prev) {
        this.personas = this.personas.map((p) => (p.id === prev.id ? prev : p));
      }
      throw err;
    }
  }

  async deletePersona(id: string): Promise<void> {
    const persona = this.personas.find((p) => p.id === id);
    try {
      if (persona?.avatar_path) {
        try {
          await apiDeleteAvatar(persona.avatar_path);
        } catch (err) {
          console.warn("Failed to delete avatar file:", err);
        }
      }
      await apiDeletePersona(id);
      this.personas = this.personas.filter((p) => p.id !== id);
    } catch (err) {
      console.error("Failed to delete persona:", err);
      throw err;
    }
  }

  // ── Lorebook CRUD ───────────────────────────────────────────────────────

  async createLorebook(name: string): Promise<Lorebook> {
    const lorebook: Lorebook = {
      id: crypto.randomUUID(),
      name,
      description: "",
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    try {
      const saved = await apiSaveLorebook(lorebook);
      this.lorebooks = [saved, ...this.lorebooks];
      return saved;
    } catch (err) {
      console.error("Failed to create lorebook:", err);
      throw err;
    }
  }

  async updateLorebook(lorebook: Lorebook): Promise<Lorebook> {
    const prev = this.lorebooks.find((l) => l.id === lorebook.id);
    try {
      const saved = await apiSaveLorebook({
        ...lorebook,
        updated_at: new Date().toISOString(),
      });
      this.lorebooks = this.lorebooks.map((l) =>
        l.id === saved.id ? saved : l,
      );
      return saved;
    } catch (err) {
      console.error("Failed to update lorebook:", err);
      if (prev) {
        this.lorebooks = this.lorebooks.map((l) =>
          l.id === prev.id ? prev : l,
        );
      }
      throw err;
    }
  }

  async deleteLorebook(id: string): Promise<void> {
    try {
      await apiDeleteLorebook(id);
      this.lorebooks = this.lorebooks.filter((l) => l.id !== id);
    } catch (err) {
      console.error("Failed to delete lorebook:", err);
      throw err;
    }
  }

  // ── Character-lorebook junction ────────────────────────────────────────

  async getCharacterLorebooks(characterId: string): Promise<Lorebook[]> {
    try {
      return await apiGetCharacterLorebooks(characterId);
    } catch (err) {
      console.error("Failed to load character lorebooks:", err);
      throw err;
    }
  }

  async assignLorebookToCharacter(
    characterId: string,
    lorebookId: string,
    sortOrder: number,
  ): Promise<void> {
    try {
      await apiAssignLorebookToCharacter(characterId, lorebookId, sortOrder);
    } catch (err) {
      console.error("Failed to assign lorebook to character:", err);
      throw err;
    }
  }

  async removeLorebookFromCharacter(
    characterId: string,
    lorebookId: string,
  ): Promise<void> {
    try {
      await apiRemoveLorebookFromCharacter(characterId, lorebookId);
    } catch (err) {
      console.error("Failed to remove lorebook from character:", err);
      throw err;
    }
  }

  // ── Regex scripts ────────────────────────────────────────────────────

  async loadRegexScripts(characterId: string): Promise<void> {
    if (this.regex_scripts.has(characterId)) return; // cached
    try {
      const scripts = await apiGetRegexScripts(characterId);
      this.regex_scripts.set(characterId, scripts);
      // Trigger reactivity by creating new Map
      this.regex_scripts = new Map(this.regex_scripts);
    } catch (err) {
      console.error("Failed to load regex scripts:", err);
    }
  }

  async saveRegexScript(script: RegexScript): Promise<RegexScript> {
    try {
      const saved = await apiSaveRegexScript(script);
      const charScripts = this.regex_scripts.get(saved.character_id) ?? [];
      const updated = charScripts.map((s) => (s.id === saved.id ? saved : s));
      if (!charScripts.find((s) => s.id === saved.id)) {
        updated.push(saved);
      }
      this.regex_scripts.set(saved.character_id, updated);
      this.regex_scripts = new Map(this.regex_scripts);
      return saved;
    } catch (err) {
      console.error("Failed to save regex script:", err);
      throw err;
    }
  }

  async deleteRegexScript(id: string, characterId: string): Promise<void> {
    try {
      await apiDeleteRegexScript(id);
      const charScripts = this.regex_scripts.get(characterId) ?? [];
      this.regex_scripts.set(
        characterId,
        charScripts.filter((s) => s.id !== id),
      );
      this.regex_scripts = new Map(this.regex_scripts);
    } catch (err) {
      console.error("Failed to delete regex script:", err);
      throw err;
    }
  }

  async testRegexPattern(
    pattern: string,
    replacePattern: string,
    flags: string,
    sampleText: string,
  ): Promise<RegexTestResult> {
    return await apiApplySingleRegex(pattern, replacePattern, flags, sampleText);
  }

  // ── Conversation lorebooks ──────────────────────────────────────────

  async loadConversationLorebooks(conversationId: string): Promise<void> {
    try {
      this.conversation_lorebooks =
        await apiGetConversationLorebooks(conversationId);
    } catch (err) {
      console.error("Failed to load conversation lorebooks:", err);
      this.conversation_lorebooks = [];
    }
  }

  async loadCharacterLorebooks(characterId: string): Promise<void> {
    try {
      this.character_lorebooks =
        await apiGetCharacterLorebooks(characterId);
    } catch (err) {
      console.error("Failed to load character lorebooks:", err);
      this.character_lorebooks = [];
    }
  }

  async toggleConversationLorebook(
    conversationId: string,
    lorebookId: string,
  ): Promise<void> {
    const isAssigned = this.conversation_lorebooks.some(
      (l) => l.id === lorebookId,
    );
    try {
      if (isAssigned) {
        await apiRemoveLorebookFromConversation(conversationId, lorebookId);
        this.conversation_lorebooks = this.conversation_lorebooks.filter(
          (l) => l.id !== lorebookId,
        );
      } else {
        const sortOrder = this.conversation_lorebooks.length;
        await apiAssignLorebookToConversation(
          conversationId,
          lorebookId,
          sortOrder,
        );
        const lorebook = this.lorebooks.find((l) => l.id === lorebookId);
        if (lorebook) {
          this.conversation_lorebooks = [
            ...this.conversation_lorebooks,
            lorebook,
          ];
        }
      }
    } catch (err) {
      console.error("Failed to toggle conversation lorebook:", err);
    }
  }

  // ── Avatar upload ──────────────────────────────────────────────────────

  async uploadAvatar(id: string, sourcePath: string): Promise<string> {
    try {
      return await apiUploadAvatar(id, sourcePath);
    } catch (err) {
      console.error("Failed to upload avatar:", err);
      throw err;
    }
  }

  // ── Card import/export ────────────────────────────────────────────────

  /** Open a file picker for card import, parse the file via backend,
   *  and return the character data for the editor to consume. */
  async importCharacterFromFile(): Promise<CharacterCard> {
    const filePath = await open({
      multiple: false,
      filters: [{ name: "Character Cards", extensions: ["png", "json"] }],
    });

    if (!filePath) {
      throw new Error("cancelled");
    }

    // open() returns string | string[] | null. With multiple: false, it's string | null.
    const path = typeof filePath === "string" ? filePath : filePath;

    return await apiImportCard(path);
  }

  /** Export a character card as a TavernAI V2 PNG file.
   *  Opens a native save dialog and writes the file via backend. */
  async exportCharacter(id: string): Promise<void> {
    const character = this.characters.find((c) => c.id === id);
    if (!character) throw new Error("Character not found");

    const savePath = await save({
      defaultPath: `${character.name}.png`,
      filters: [{ name: "PNG Image", extensions: ["png"] }],
    });

    if (!savePath) return; // User cancelled

    await apiExportCard(id, savePath);
  }

  // ── Provider CRUD ──────────────────────────────────────────────────────

  async createProvider(
    data: Omit<Provider, "id" | "created_at" | "updated_at" | "has_api_key"> & {
      id?: string;
      api_key?: string;
    },
  ): Promise<Provider> {
    const provider: Provider = {
      id: data.id ?? crypto.randomUUID(),
      name: data.name,
      provider_type: data.provider_type,
      base_url: data.base_url,
      model_name: data.model_name,
      models: data.models ?? [],
      context_window_size: data.context_window_size,
      default_params: data.default_params,
      has_api_key: false,
      enabled: data.enabled ?? true,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    try {
      const saved = await apiSaveProvider(provider);
      if (data.api_key) {
        try {
          await apiSaveApiKey(saved.id, data.api_key);
          saved.has_api_key = true;
        } catch (err) {
          console.error("Failed to save API key to keyring:", err);
        }
      }
      this.providers = [saved, ...this.providers];
      return saved;
    } catch (err) {
      console.error("Failed to create provider:", err);
      throw err;
    }
  }

  async updateProvider(provider: Provider, apiKey?: string): Promise<Provider> {
    const prev = this.providers.find((p) => p.id === provider.id);
    try {
      const saved = await apiSaveProvider({
        ...provider,
        updated_at: new Date().toISOString(),
      });
      if (apiKey !== undefined) {
        try {
          await apiSaveApiKey(saved.id, apiKey);
          saved.has_api_key = true;
        } catch (err) {
          console.error("Failed to update API key:", err);
        }
      }
      this.providers = this.providers.map((p) =>
        p.id === saved.id ? saved : p,
      );
      return saved;
    } catch (err) {
      console.error("Failed to update provider:", err);
      if (prev) {
        this.providers = this.providers.map((p) =>
          p.id === prev.id ? prev : p,
        );
      }
      throw err;
    }
  }

  async deleteProvider(id: string): Promise<void> {
    try {
      await apiDeleteProvider(id);
      this.providers = this.providers.filter((p) => p.id !== id);
    } catch (err) {
      console.error("Failed to delete provider:", err);
      throw err;
    }
  }

  async updateConversationProvider(
    providerId: string | null,
    modelName: string | null,
  ): Promise<void> {
    if (!this.active_conversation_id) return;
    const conv = this.active_conversation;
    const prevProviderId = conv?.provider_id ?? null;
    const prevModelName = conv?.model_name ?? null;
    // Optimistically update local state
    this.conversations = this.conversations.map((c) =>
      c.id === this.active_conversation_id
        ? {
            ...c,
            provider_id: providerId ?? undefined,
            model_name: modelName ?? undefined,
          }
        : c,
    );
    try {
      await apiUpdateConversationProvider(
        this.active_conversation_id,
        providerId,
        modelName,
      );
    } catch (err) {
      console.error("Failed to persist provider/model switch:", err);
      // Revert on failure
      this.conversations = this.conversations.map((c) =>
        c.id === this.active_conversation_id
          ? {
              ...c,
              provider_id: prevProviderId ?? undefined,
              model_name: prevModelName ?? undefined,
            }
          : c,
      );
    }
  }

  async deleteApiKey(providerId: string): Promise<void> {
    try {
      await apiDeleteApiKey(providerId);
      this.providers = this.providers.map((p) =>
        p.id === providerId ? { ...p, has_api_key: false } : p,
      );
    } catch (err) {
      console.error("Failed to delete API key:", err);
      throw err;
    }
  }

  async checkKeyring(): Promise<void> {
    try {
      this.keyring_available = await apiCheckKeyringAvailable();
    } catch (err) {
      this.keyring_available = false;
      console.error("Failed to check keyring availability:", err);
    }
  }

  // ── Streaming chat actions ───────────────────────────────────────────

  /** Send a user message and stream an AI response via Tauri Channel. */
  async startChat(content: string): Promise<void> {
    if (!this.active_conversation_id) return;
    const conv = this.active_conversation;
    if (!conv) return;

    // Check provider is configured
    if (!conv.provider_id) {
      const errMsg: Message = {
        id: crypto.randomUUID(),
        conversation_id: this.active_conversation_id,
        role: "assistant",
        content:
          "No provider configured. Select a provider and model from the header.",
        sort_order: (this.messages.at(-1)?.sort_order ?? -1) + 1,
        timestamp: new Date().toISOString(),
        is_streaming: false,
        branch_id: this.active_branch_id,
        metadata: {
          stop_reason: "error",
          error_type: "no_provider_configured",
        },
      };
      this.appendMessage(errMsg);
      return;
    }

    const now = new Date().toISOString();
    const nextSortOrder = (this.messages.at(-1)?.sort_order ?? -1) + 1;

    // Create user message (frontend pre-generates id and sort_order)
    const userMsg: Message = {
      id: crypto.randomUUID(),
      conversation_id: this.active_conversation_id,
      role: "user",
      content,
      sort_order: nextSortOrder,
      timestamp: now,
      branch_id: this.active_branch_id,
    };
    this.appendMessage(userMsg);

    // Create AI placeholder
    const aiId = crypto.randomUUID();
    const aiMsg: Message = {
      id: aiId,
      conversation_id: this.active_conversation_id,
      role: "assistant",
      content: "",
      sort_order: nextSortOrder + 1,
      timestamp: now,
      is_streaming: true,
      branch_id: this.active_branch_id,
      metadata: {
        model: conv.model_name,
      },
    };
    this.appendMessage(aiMsg);
    this.is_generating = true;

    // Set up Channel for streaming
    const onEvent = new Channel<StreamChunk>();
    let accumulated = "";

    onEvent.onmessage = (chunk: StreamChunk) => {
      switch (chunk.event) {
        case "token":
          accumulated += chunk.data.content;
          this.updateStreamingMessage(aiId, accumulated);
          break;
        case "complete": {
          this.updateStreamingMessage(
            aiId,
            chunk.data.content || accumulated,
            true,
          );
          // Update metadata with stop reason
          this.messages = this.messages.map((m) =>
            m.id === aiId
              ? {
                  ...m,
                  metadata: {
                    ...m.metadata,
                    stop_reason: chunk.data
                      .stopReason as Message["metadata"] extends {
                      stop_reason?: infer S;
                    }
                      ? S
                      : never,
                  },
                }
              : m,
          );
          // Update context budget
          this.updateContextBudget({
            tokensUsed: chunk.data.tokensUsed,
            tokensLimit: chunk.data.tokensLimit,
            truncatedCount: chunk.data.truncatedCount,
          });
          // Update lorebook details from complete event
          if (chunk.data.lorebookDetails) {
            this.lorebook_details = chunk.data.lorebookDetails;
          }
          // Persist the completed AI message to the database
          const completedMsg = this.messages.find((m) => m.id === aiId);
          if (completedMsg) {
            apiSaveMessage(completedMsg).catch((err) =>
              console.error("Failed to save AI message:", err),
            );
            this.updateConversationPreview(
              completedMsg.conversation_id,
              completedMsg.content,
            );
          }
          break;
        }
        case "error":
          this.messages = this.messages.map((m) =>
            m.id === aiId
              ? {
                  ...m,
                  content: chunk.data.error,
                  is_streaming: false,
                  metadata: {
                    ...m.metadata,
                    stop_reason: "error" as const,
                    error_type: chunk.data.errorType,
                  },
                }
              : m,
          );
          this.is_generating = false;
          break;
      }
    };

    try {
      await apiStartChat(
        this.active_conversation_id,
        content,
        userMsg.id,
        userMsg.sort_order,
        aiId,
        onEvent,
        this.active_branch_id,
      );
    } catch (err) {
      console.error("startChat invoke failed:", err);
      const errorContent =
        err instanceof Error
          ? err.message
          : typeof err === "string"
            ? err
            : "Failed to start generation.";
      this.messages = this.messages.map((m) =>
        m.id === aiId
          ? {
              ...m,
              content: errorContent,
              is_streaming: false,
              metadata: {
                ...m.metadata,
                stop_reason: "error" as const,
                error_type: "provider_error",
              },
            }
          : m,
      );
      this.is_generating = false;
    }
  }

  /** Stop the active AI generation. */
  async stopGeneration(): Promise<void> {
    try {
      await apiStopGeneration();
    } catch (err) {
      console.error("stopGeneration failed:", err);
    }
  }

  /** Regenerate the last AI response. */
  async regenerate(): Promise<void> {
    if (!this.active_conversation_id) return;

    // Remove the last AI message from local state
    const lastAiIdx = this.messages.findLastIndex(
      (m) => m.role === "assistant" && m.metadata?.stop_reason !== "error",
    );
    if (lastAiIdx === -1) return;

    this.messages = this.messages.filter((_, i) => i !== lastAiIdx);

    // Create AI placeholder
    const aiId = crypto.randomUUID();
    const nextSortOrder = (this.messages.at(-1)?.sort_order ?? -1) + 1;
    const aiMsg: Message = {
      id: aiId,
      conversation_id: this.active_conversation_id,
      role: "assistant",
      content: "",
      sort_order: nextSortOrder,
      timestamp: new Date().toISOString(),
      is_streaming: true,
      branch_id: this.active_branch_id,
      metadata: { model: this.active_conversation?.model_name },
    };
    this.appendMessage(aiMsg);
    this.is_generating = true;

    // Set up Channel
    const onEvent = new Channel<StreamChunk>();
    let accumulated = "";

    onEvent.onmessage = (chunk: StreamChunk) => {
      switch (chunk.event) {
        case "token":
          accumulated += chunk.data.content;
          this.updateStreamingMessage(aiId, accumulated);
          break;
        case "complete": {
          this.updateStreamingMessage(
            aiId,
            chunk.data.content || accumulated,
            true,
          );
          this.messages = this.messages.map((m) =>
            m.id === aiId
              ? {
                  ...m,
                  metadata: {
                    ...m.metadata,
                    stop_reason: chunk.data
                      .stopReason as Message["metadata"] extends {
                      stop_reason?: infer S;
                    }
                      ? S
                      : never,
                  },
                }
              : m,
          );
          // Update context budget
          this.updateContextBudget({
            tokensUsed: chunk.data.tokensUsed,
            tokensLimit: chunk.data.tokensLimit,
            truncatedCount: chunk.data.truncatedCount,
          });
          // Update lorebook details from complete event
          if (chunk.data.lorebookDetails) {
            this.lorebook_details = chunk.data.lorebookDetails;
          }
          // Persist the regenerated AI message to the database
          const regeneratedMsg = this.messages.find((m) => m.id === aiId);
          if (regeneratedMsg) {
            apiSaveMessage(regeneratedMsg).catch((err) =>
              console.error("Failed to save regenerated AI message:", err),
            );
            this.updateConversationPreview(
              regeneratedMsg.conversation_id,
              regeneratedMsg.content,
            );
          }
          break;
        }
        case "error":
          this.messages = this.messages.map((m) =>
            m.id === aiId
              ? {
                  ...m,
                  content: chunk.data.error,
                  is_streaming: false,
                  metadata: {
                    ...m.metadata,
                    stop_reason: "error" as const,
                    error_type: chunk.data.errorType,
                  },
                }
              : m,
          );
          this.is_generating = false;
          break;
      }
    };

    try {
      await apiRegenerate(
        this.active_conversation_id,
        aiId,
        onEvent,
        this.active_branch_id,
      );
    } catch (err) {
      console.error("regenerate failed:", err);
      const errorContent =
        err instanceof Error
          ? err.message
          : typeof err === "string"
            ? err
            : "Regeneration failed.";
      this.messages = this.messages.map((m) =>
        m.id === aiId
          ? {
              ...m,
              content: errorContent,
              is_streaming: false,
              metadata: {
                ...m.metadata,
                stop_reason: "error" as const,
                error_type: "provider_error",
              },
            }
          : m,
      );
      this.is_generating = false;
    }
  }

  /** Edit a message and resend from that point. */
  async editResend(messageId: string, newContent: string): Promise<void> {
    if (!this.active_conversation_id) return;

    // Find the message and update its content locally
    const msg = this.messages.find((m) => m.id === messageId);
    if (!msg) return;

    // Save state for rollback on failure
    const prevMessages = [...this.messages];

    // Update content locally
    this.messages = this.messages.map((m) =>
      m.id === messageId ? { ...m, content: newContent } : m,
    );

    // Remove all subsequent messages
    this.messages = this.messages.filter((m) => m.sort_order <= msg.sort_order);

    // Create AI placeholder
    const aiId = crypto.randomUUID();
    const nextSortOrder = msg.sort_order + 1;
    const aiMsg: Message = {
      id: aiId,
      conversation_id: this.active_conversation_id,
      role: "assistant",
      content: "",
      sort_order: nextSortOrder,
      timestamp: new Date().toISOString(),
      is_streaming: true,
      branch_id: this.active_branch_id,
      metadata: { model: this.active_conversation?.model_name },
    };
    this.appendMessage(aiMsg);
    this.is_generating = true;

    // Set up Channel
    const onEvent = new Channel<StreamChunk>();
    let accumulated = "";

    onEvent.onmessage = (chunk: StreamChunk) => {
      switch (chunk.event) {
        case "token":
          accumulated += chunk.data.content;
          this.updateStreamingMessage(aiId, accumulated);
          break;
        case "complete": {
          this.updateStreamingMessage(
            aiId,
            chunk.data.content || accumulated,
            true,
          );
          this.messages = this.messages.map((m) =>
            m.id === aiId
              ? {
                  ...m,
                  metadata: {
                    ...m.metadata,
                    stop_reason: chunk.data
                      .stopReason as Message["metadata"] extends {
                      stop_reason?: infer S;
                    }
                      ? S
                      : never,
                  },
                }
              : m,
          );
          // Update context budget
          this.updateContextBudget({
            tokensUsed: chunk.data.tokensUsed,
            tokensLimit: chunk.data.tokensLimit,
            truncatedCount: chunk.data.truncatedCount,
          });
          // Update lorebook details from complete event
          if (chunk.data.lorebookDetails) {
            this.lorebook_details = chunk.data.lorebookDetails;
          }
          // Persist the edited AI message to the database
          const editedMsg = this.messages.find((m) => m.id === aiId);
          if (editedMsg) {
            apiSaveMessage(editedMsg).catch((err) =>
              console.error("Failed to save edited AI message:", err),
            );
            this.updateConversationPreview(
              editedMsg.conversation_id,
              editedMsg.content,
            );
          }
          break;
        }
        case "error":
          this.messages = this.messages.map((m) =>
            m.id === aiId
              ? {
                  ...m,
                  content: chunk.data.error,
                  is_streaming: false,
                  metadata: {
                    ...m.metadata,
                    stop_reason: "error" as const,
                    error_type: chunk.data.errorType,
                  },
                }
              : m,
          );
          this.is_generating = false;
          break;
      }
    };

    try {
      await apiEditAndResend(
        messageId,
        newContent,
        this.active_conversation_id,
        aiId,
        onEvent,
        this.active_branch_id,
      );
    } catch (err) {
      console.error("editResend failed:", err);
      // Rollback to previous state (removes AI placeholder too)
      this.messages = prevMessages;
      this.is_generating = false;
      throw err;
    }
  }
}

export const app = new AppState();
