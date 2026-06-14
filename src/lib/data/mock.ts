/**
 * Mock data for dev-mode (frontend-only) development.
 *
 * Used by api.ts when `__TAURI_INTERNALS__` is not present (i.e. running
 * `bun dev` without the Tauri backend). These are NOT a replacement for the
 * real backend implementations — they exist purely for UI development.
 */

import type { Channel } from "@tauri-apps/api/core";
import type {
  GenerateCharacterChunk,
  GeneratedCharacterData,
} from "$lib/types";

export const MOCK_GENERATED_CHARACTER: GeneratedCharacterData = {
  name: "Aria Nightwhisper",
  description:
    "A mysterious half-elf archivist who tends the Forbidden Stacks of the Grand Athenaeum. She speaks in riddles and forgotten tongues, guarding knowledge that reality has tried to unwrite.",
  personality:
    "Eccentric and melancholic. Aria oscillates between playful teasing and somber introspection. She hoards secrets like coins and delights in being the only one who remembers things that never happened.",
  scenario:
    "You've wandered into the Forbidden Stacks after hours. The candles are still burning — which shouldn't be possible, since no one has been here in decades. Aria is waiting, as if she knew you'd come.",
  first_mes:
    "*The candlelight doesn't flicker as you enter — it bends toward you, curious.*\n\nAnother lost soul stumbling where the library's fingers are longest. *She looks up from a book that seems to be reading itself.* Tell me, seeker — are you looking for something, or running from it? The Stacks accept both kinds.",
  mes_example:
    "<START>\n{{user}}: What's this book?\n{{char}}: *Aria's eyes narrow with something between amusement and warning.* That particular volume has been trying to escape for three centuries. I'd put it down before it gets ideas — or put it back before it puts you down. The Stacks have a sense of humor, and it is not kind.",
  system_prompt:
    "Write as Aria Nightwhisper, the half-elf archivist. Speak cryptically but warmly. Reference forgotten knowledge and impossible books.",
  post_history_instructions: "",
  creator_notes: "Generated as a demo character.",
  tags: ["fantasy", "elf", "librarian", "mysterious"],
  alternate_greetings: [],
};

export function mockGenerateCharacter(
  _idea: string,
  onEvent: Channel<GenerateCharacterChunk>,
): Promise<void> {
  const jsonPreview = JSON.stringify(MOCK_GENERATED_CHARACTER, null, 2);
  const chunkSize = 8;
  let offset = 0;

  return new Promise((resolve) => {
    function sendChunk() {
      if (offset < jsonPreview.length) {
        const content = jsonPreview.slice(offset, offset + chunkSize);
        offset += chunkSize;
        onEvent.onmessage({ event: "token", data: { content } });
        setTimeout(sendChunk, 30 + Math.random() * 40);
      } else {
        onEvent.onmessage({
          event: "complete",
          data: { character: MOCK_GENERATED_CHARACTER },
        });
        resolve();
      }
    }
    setTimeout(sendChunk, 200);
  });
}
