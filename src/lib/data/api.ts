/**
 * Typed API layer — wraps Tauri v2 invoke() calls for all 16 backend commands.
 *
 * Command names use snake_case matching Rust function names.
 * Arguments use camelCase (Tauri v2 auto-converts to snake_case for Rust).
 */

import { invoke, type Channel } from "@tauri-apps/api/core";
import type {
  CharacterCard,
  Conversation,
  Message,
  MessageBranch,
  SearchResult,
  UserPersona,
  Provider,
  ProviderType,
  ProviderModelEntry,
  GenerationPreset,
  Lorebook,
  LorebookEntry,
  KeywordTestResult,
  RegexScript,
  RegexTestResult,
  StreamChunk,
  GenerateCharacterChunk,
} from "$lib/types";
import { mockGenerateCharacter } from "$lib/data/mock";

// ── Dev-mode detection ──────────────────────────────────────────────────────────

const IS_DEV =
  typeof window !== "undefined" && !("__TAURI_INTERNALS__" in window);

// ── Characters ────────────────────────────────────────────────────────────────

export async function getCharacters(): Promise<CharacterCard[]> {
  return invoke<CharacterCard[]>("get_characters");
}

export async function saveCharacter(
  character: CharacterCard,
): Promise<CharacterCard> {
  return invoke<CharacterCard>("save_character", { character });
}

export async function deleteCharacter(id: string): Promise<void> {
  return invoke<void>("delete_character", { id });
}

// ── Personas ──────────────────────────────────────────────────────────────────

export async function getPersonas(): Promise<UserPersona[]> {
  return invoke<UserPersona[]>("get_personas");
}

export async function savePersona(persona: UserPersona): Promise<UserPersona> {
  return invoke<UserPersona>("save_persona", { persona });
}

export async function deletePersona(id: string): Promise<void> {
  return invoke<void>("delete_persona", { id });
}

// ── Conversations ─────────────────────────────────────────────────────────────

export async function getConversations(): Promise<Conversation[]> {
  return invoke<Conversation[]>("get_conversations");
}

export async function startConversation(
  characterId: string,
): Promise<Conversation> {
  return invoke<Conversation>("start_conversation", { characterId });
}

export async function deleteConversation(id: string): Promise<void> {
  return invoke<void>("delete_conversation", { id });
}

export async function renameConversation(
  id: string,
  title: string,
): Promise<void> {
  return invoke<void>("rename_conversation", { id, title });
}

// ── Messages ──────────────────────────────────────────────────────────────────

export async function getMessages(
  conversationId: string,
  branchId?: string | null,
): Promise<Message[]> {
  return invoke<Message[]>("get_messages", {
    conversationId,
    branchId: branchId ?? null,
  });
}

export async function saveMessage(message: Message): Promise<void> {
  return invoke<void>("save_message", { message });
}

export async function deleteMessage(id: string): Promise<void> {
  return invoke<void>("delete_message", { id });
}

// ── Conversation persona ──────────────────────────────────────────────────────

export async function updateConversationPersona(
  conversationId: string,
  personaId: string | null,
): Promise<void> {
  return invoke<void>("update_conversation_persona", {
    conversationId,
    personaId,
  });
}

export async function updateConversationProvider(
  id: string,
  providerId: string | null,
  modelName: string | null,
): Promise<void> {
  return invoke<void>("update_conversation_provider", {
    id,
    providerId,
    modelName,
  });
}

// ── Presets ────────────────────────────────────────────────────────────────────

export async function getPresets(): Promise<GenerationPreset[]> {
  return invoke<GenerationPreset[]>("get_presets");
}

export async function savePreset(
  preset: GenerationPreset,
): Promise<GenerationPreset> {
  return invoke<GenerationPreset>("save_preset", { preset });
}

export async function deletePreset(id: string): Promise<void> {
  return invoke<void>("delete_preset", { id });
}

export async function updateConversationGeneration(
  conversationId: string,
  generationPresetId: string | null,
  overrideMaxTokens: number | null,
): Promise<void> {
  return invoke<void>("update_conversation_generation", {
    conversationId,
    generationPresetId,
    overrideMaxTokens,
  });
}

// ── Lorebooks ──────────────────────────────────────────────────────────────────

export async function getLorebooks(): Promise<Lorebook[]> {
  return invoke<Lorebook[]>("get_lorebooks");
}

export async function saveLorebook(lorebook: Lorebook): Promise<Lorebook> {
  return invoke<Lorebook>("save_lorebook", { lorebook });
}

export async function deleteLorebook(id: string): Promise<void> {
  return invoke<void>("delete_lorebook", { id });
}

export async function importLorebook(filePath: string): Promise<Lorebook> {
  return invoke<Lorebook>("import_lorebook", { filePath });
}

// ── Lorebook entries ───────────────────────────────────────────────────────────

export async function getLorebookEntries(
  lorebookId: string,
): Promise<LorebookEntry[]> {
  return invoke<LorebookEntry[]>("get_lorebook_entries", { lorebookId });
}

export async function saveLorebookEntry(
  entry: LorebookEntry,
): Promise<LorebookEntry> {
  return invoke<LorebookEntry>("save_lorebook_entry", { entry });
}

export async function deleteLorebookEntry(id: string): Promise<void> {
  return invoke<void>("delete_lorebook_entry", { id });
}

// ── Character-lorebook junction ────────────────────────────────────────────────

export async function assignLorebookToCharacter(
  characterId: string,
  lorebookId: string,
  sortOrder: number,
): Promise<void> {
  return invoke<void>("assign_lorebook_to_character", {
    characterId,
    lorebookId,
    sortOrder,
  });
}

export async function removeLorebookFromCharacter(
  characterId: string,
  lorebookId: string,
): Promise<void> {
  return invoke<void>("remove_lorebook_from_character", {
    characterId,
    lorebookId,
  });
}

export async function getCharacterLorebooks(
  characterId: string,
): Promise<Lorebook[]> {
  return invoke<Lorebook[]>("get_character_lorebooks", { characterId });
}

// ── Avatars ────────────────────────────────────────────────────────────────────

export async function uploadAvatar(
  id: string,
  sourcePath: string,
): Promise<string> {
  return invoke<string>("upload_avatar", { id, sourcePath });
}

export async function deleteAvatar(path: string): Promise<void> {
  return invoke<void>("delete_avatar", { path });
}

// ── Keyword testing ───────────────────────────────────────────────────────────

export async function testLorebookKeywords(
  keys: string[],
  secondaryKeys: string[],
  text: string,
  activationState?: string | null,
): Promise<KeywordTestResult> {
  return invoke<KeywordTestResult>("test_lorebook_keywords", {
    keys,
    secondaryKeys,
    text,
    activationState: activationState ?? null,
  });
}

// ── Providers ───────────────────────────────────────────────────────────────

export async function getProviders(): Promise<Provider[]> {
  return invoke<Provider[]>("get_providers");
}

export async function saveProvider(provider: Provider): Promise<Provider> {
  return invoke<Provider>("save_provider", { provider });
}

export async function deleteProvider(id: string): Promise<void> {
  return invoke<void>("delete_provider", { id });
}

/**
 * Detect available models from a provider's Models API.
 *
 * No dev-mode mock: this hits a real network endpoint, so a mock would mislead.
 * Throws if called without the Tauri backend (consistent with keyring commands).
 */
export async function listProviderModels(args: {
  providerType: ProviderType;
  baseUrl?: string;
  providerId?: string;
  apiKey?: string;
}): Promise<ProviderModelEntry[]> {
  return invoke<ProviderModelEntry[]>("list_provider_models", {
    providerType: args.providerType,
    baseUrl: args.baseUrl ?? null,
    providerId: args.providerId ?? null,
    apiKey: args.apiKey ?? null,
  });
}

// ── Keyring ──────────────────────────────────────────────────────────────────

export async function saveApiKey(
  providerId: string,
  apiKey: string,
): Promise<void> {
  return invoke<void>("save_api_key", { providerId, apiKey });
}

export async function deleteApiKey(providerId: string): Promise<void> {
  return invoke<void>("delete_api_key", { providerId });
}

export async function checkKeyringAvailable(): Promise<boolean> {
  return invoke<boolean>("check_keyring_available");
}

// ── Chat streaming ────────────────────────────────────────────────────────────

export async function startChat(
  conversationId: string,
  content: string,
  messageId: string,
  sortOrder: number,
  aiMessageId: string,
  onEvent: Channel<StreamChunk>,
  branchId?: string | null,
  steering?: string,
): Promise<void> {
  return invoke<void>("start_chat", {
    conversationId,
    content,
    messageId,
    sortOrder,
    aiMessageId,
    branchId: branchId ?? null,
    channel: onEvent,
    steering: steering ?? null,
  });
}

export async function stopGeneration(): Promise<void> {
  return invoke<void>("stop_generation");
}

export async function regenerate(
  conversationId: string,
  aiMessageId: string,
  onEvent: Channel<StreamChunk>,
  branchId?: string | null,
): Promise<string | null> {
  return invoke<string | null>("regenerate", {
    conversationId,
    aiMessageId,
    branchId: branchId ?? null,
    channel: onEvent,
  });
}

export async function editAndResend(
  messageId: string,
  newContent: string,
  conversationId: string,
  aiMessageId: string,
  onEvent: Channel<StreamChunk>,
  branchId?: string | null,
): Promise<void> {
  return invoke<void>("edit_and_resend", {
    messageId,
    newContent,
    conversationId,
    aiMessageId,
    branchId: branchId ?? null,
    channel: onEvent,
  });
}

// ── Card I/O ────────────────────────────────────────────────────────────────

export async function importCard(filePath: string): Promise<CharacterCard> {
  return invoke<CharacterCard>("import_card", { filePath });
}

export async function exportCard(
  characterId: string,
  savePath: string,
): Promise<void> {
  return invoke<void>("export_card", { characterId, savePath });
}

// ── Branches ────────────────────────────────────────────────────────────────────

export async function forkConversation(
  conversationId: string,
  parentMessageId: string,
): Promise<MessageBranch> {
  return invoke<MessageBranch>("fork_conversation", {
    conversationId,
    parentMessageId,
  });
}

export async function getBranchesForConversation(
  conversationId: string,
): Promise<MessageBranch[]> {
  return invoke<MessageBranch[]>("get_branches_for_conversation", {
    conversationId,
  });
}

export async function deleteBranch(branchId: string): Promise<void> {
  return invoke<void>("delete_branch", { branchId });
}

export async function searchConversations(
  query: string,
): Promise<Conversation[]> {
  return invoke<Conversation[]>("search_conversations", { query });
}

export async function searchMessages(query: string): Promise<SearchResult[]> {
  return invoke<SearchResult[]>("search_messages", { query });
}

// ── Regex scripts ──────────────────────────────────────────────────────────────

export async function getRegexScripts(
  characterId: string,
): Promise<RegexScript[]> {
  return invoke<RegexScript[]>("get_regex_scripts", { characterId });
}

export async function saveRegexScript(
  script: RegexScript,
): Promise<RegexScript> {
  return invoke<RegexScript>("save_regex_script", { script });
}

export async function deleteRegexScript(id: string): Promise<void> {
  return invoke<void>("delete_regex_script", { id });
}

export async function applySingleRegex(
  pattern: string,
  replacePattern: string,
  flags: string,
  sampleText: string,
): Promise<RegexTestResult> {
  return invoke<RegexTestResult>("apply_single_regex", {
    pattern,
    replacePattern,
    flags,
    sampleText,
  });
}

export async function validateRegexPattern(
  pattern: string,
  flags: string,
): Promise<void> {
  return invoke<void>("validate_regex_pattern", { pattern, flags });
}

// ── Conversation-lorebook junction ────────────────────────────────────────────

export async function getConversationLorebooks(
  conversationId: string,
): Promise<Lorebook[]> {
  return invoke<Lorebook[]>("get_conversation_lorebooks", {
    conversationId,
  });
}

export async function assignLorebookToConversation(
  conversationId: string,
  lorebookId: string,
  sortOrder: number,
): Promise<void> {
  return invoke<void>("assign_lorebook_to_conversation", {
    conversationId,
    lorebookId,
    sortOrder,
  });
}

export async function removeLorebookFromConversation(
  conversationId: string,
  lorebookId: string,
): Promise<void> {
  return invoke<void>("remove_lorebook_from_conversation", {
    conversationId,
    lorebookId,
  });
}

// ── App settings ──────────────────────────────────────────────────────────────

export async function getAppSetting(key: string): Promise<string | null> {
  return invoke<string | null>("get_app_setting", { key });
}

export async function setAppSetting(key: string, value: string): Promise<void> {
  return invoke<void>("set_app_setting", { key, value });
}

// ── Character generation ──────────────────────────────────────────────────────

export async function stopCharacterGeneration(): Promise<void> {
  return invoke<void>("stop_character_generation");
}

export async function generateCharacter(
  idea: string,
  providerId: string,
  modelName: string,
  onEvent: Channel<GenerateCharacterChunk>,
): Promise<void> {
  if (IS_DEV) {
    return mockGenerateCharacter(idea, onEvent);
  }
  return invoke<void>("generate_character", {
    idea,
    providerId,
    modelName,
    channel: onEvent,
  });
}
