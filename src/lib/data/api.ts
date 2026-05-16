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
  UserPersona,
  Provider,
  ProviderModelEntry,
  GenerationPreset,
  Lorebook,
  LorebookEntry,
  KeywordTestResult,
  StreamChunk,
} from "$lib/types";

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

export async function searchCharacters(
  query: string,
): Promise<CharacterCard[]> {
  return invoke<CharacterCard[]>("search_characters", { query });
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

export async function searchPersonas(query: string): Promise<UserPersona[]> {
  return invoke<UserPersona[]>("search_personas", { query });
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

export async function searchConversations(
  query: string,
): Promise<Conversation[]> {
  return invoke<Conversation[]>("search_conversations", { query });
}

// ── Messages ──────────────────────────────────────────────────────────────────

export async function getMessages(conversationId: string): Promise<Message[]> {
  return invoke<Message[]>("get_messages", { conversationId });
}

export async function saveMessage(message: Message): Promise<void> {
  return invoke<void>("save_message", { message });
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

// ── Lorebooks ──────────────────────────────────────────────────────────────────

export async function getLorebooks(): Promise<Lorebook[]> {
  return invoke<Lorebook[]>("get_lorebooks");
}

export async function getLorebook(id: string): Promise<Lorebook | null> {
  return invoke<Lorebook | null>("get_lorebook", { id });
}

export async function saveLorebook(lorebook: Lorebook): Promise<Lorebook> {
  return invoke<Lorebook>("save_lorebook", { lorebook });
}

export async function deleteLorebook(id: string): Promise<void> {
  return invoke<void>("delete_lorebook", { id });
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
): Promise<KeywordTestResult> {
  return invoke<KeywordTestResult>("test_lorebook_keywords", {
    keys,
    secondaryKeys,
    text,
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

// ── Keyring ──────────────────────────────────────────────────────────────────

export async function saveApiKey(
  providerId: string,
  apiKey: string,
): Promise<void> {
  return invoke<void>("save_api_key", { providerId, apiKey });
}

export async function hasApiKey(providerId: string): Promise<boolean> {
  return invoke<boolean>("has_api_key", { providerId });
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
  onEvent: Channel<StreamChunk>,
): Promise<void> {
  return invoke<void>("start_chat", { conversationId, content, messageId, sortOrder, onEvent });
}

export async function stopGeneration(): Promise<void> {
  return invoke<void>("stop_generation");
}

export async function regenerate(
  conversationId: string,
  onEvent: Channel<StreamChunk>,
): Promise<void> {
  return invoke<void>("regenerate", { conversationId, onEvent });
}

export async function editAndResend(
  messageId: string,
  newContent: string,
  onEvent: Channel<StreamChunk>,
): Promise<void> {
  return invoke<void>("edit_and_resend", { messageId, newContent, onEvent });
}
