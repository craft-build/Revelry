use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::io::Cursor;

use crate::domain::character::{CharacterCardV2, CharaCardV2Spec, CharacterV1};

/// Maximum allowed JSON payload size (10 MB).
const MAX_JSON_SIZE: usize = 10 * 1024 * 1024;

/// Import a character card from a JSON string.
///
/// Attempts V2 format first, then V1 with auto-migration.
/// Imported characters receive a new UUID v4 id.
/// Rejects payloads exceeding 10 MB.
pub fn import_from_json(json_str: &str) -> Result<CharacterCardV2> {
    if json_str.len() > MAX_JSON_SIZE {
        return Err(anyhow!(
            "JSON payload exceeds maximum size of {} bytes",
            MAX_JSON_SIZE
        ));
    }

    // Try V2 spec format first (spec + nested data, used in PNG tEXt chunks)
    if let Ok(spec) = serde_json::from_str::<CharaCardV2Spec>(json_str) {
        let mut card = spec.to_domain();
        normalize_card_fields(&mut card);
        return Ok(card);
    }

    // Try flat V2 format (our internal storage format)
    if let Ok(mut card) = serde_json::from_str::<CharacterCardV2>(json_str) {
        // Assign a new UUID for imported characters
        card.id = uuid::Uuid::new_v4().to_string();
        normalize_card_fields(&mut card);
        return Ok(card);
    }

    // Try V1 format
    if let Ok(v1) = serde_json::from_str::<CharacterV1>(json_str) {
        let mut card = v1.to_v2();
        normalize_card_fields(&mut card);
        return Ok(card);
    }

    Err(anyhow!(
        "Invalid character card JSON. Expected CharacterCardV2 spec, flat V2, or V1 format."
    ))
}

/// Export a character card as a pretty-printed JSON string.
pub fn export_to_json(character: &CharacterCardV2) -> Result<String> {
    Ok(serde_json::to_string_pretty(character)?)
}

/// Import a character card from PNG bytes.
///
/// Extracts the tEXt chunk with key "chara", base64-decodes it,
/// and parses the resulting JSON as a character card.
/// Returns the character and the raw PNG bytes (which IS the avatar image).
pub fn import_from_png(png_bytes: &[u8]) -> Result<(CharacterCardV2, Option<Vec<u8>>)> {
    let decoder = png::Decoder::new(Cursor::new(png_bytes));
    let mut reader = decoder.read_info()?;

    // The PNG spec allows tEXt chunks after IDAT. Many character card PNGs
    // place the "chara" tEXt chunk after the image data. We must consume the
    // image data and then read remaining chunks before checking text metadata.
    let mut img_data = vec![0u8; reader.output_buffer_size().ok_or_else(|| anyhow!("Cannot determine PNG output buffer size"))?];
    reader.next_frame(&mut img_data)?;
    reader.finish()?;

    // Look for the "chara" tEXt chunk
    let chara_value = reader
        .info()
        .uncompressed_latin1_text
        .iter()
        .find(|chunk| chunk.keyword == "chara")
        .map(|chunk| chunk.text.clone())
        .ok_or_else(|| anyhow!("No 'chara' tEXt chunk found in PNG file"))?;

    // Base64-decode the value
    let decoded = BASE64
        .decode(&chara_value)
        .map_err(|e| anyhow!("Failed to base64-decode chara chunk: {}", e))?;

    let json_str = String::from_utf8(decoded)
        .map_err(|e| anyhow!("Base64-decoded data is not valid UTF-8: {}", e))?;

    // Parse as character card
    let character = import_from_json(&json_str)?;

    Ok((character, Some(png_bytes.to_vec())))
}

/// Export a character card as PNG bytes with embedded metadata.
///
/// If avatar_bytes is provided, the PNG is read and the tEXt chunk is added.
/// If no avatar is provided, a minimal 1x1 PNG is created with the metadata.
pub fn export_to_png(
    character: &CharacterCardV2,
    avatar_bytes: Option<&[u8]>,
) -> Result<Vec<u8>> {
    // Serialize character to JSON and base64-encode
    let json = serde_json::to_string(character)?;
    let encoded = BASE64.encode(json.as_bytes());

    let mut output = Vec::new();

    if let Some(avatar) = avatar_bytes {
        // Read the existing PNG and re-encode with the tEXt chunk
        let cursor = Cursor::new(avatar);
        let decoder = png::Decoder::new(cursor);
        let mut reader = decoder.read_info()?;
        let info = reader.info().clone();
        let mut img_data = vec![0u8; reader.output_buffer_size().unwrap()];
        let _output_info = reader.next_frame(&mut img_data)?;

        let width = info.width;
        let height = info.height;

        // Re-encode with the tEXt chunk
        let mut encoder = png::Encoder::new(&mut output, width, height);
        encoder.set_color(info.color_type);
        encoder.set_depth(info.bit_depth);
        encoder.add_text_chunk("chara".to_string(), encoded)?;
        if info.trns.is_some() {
            encoder.set_trns(info.trns.as_ref().unwrap().as_ref());
        }

        let mut writer = encoder.write_header()?;
        writer.write_image_data(&img_data)?;
    } else {
        // Create a minimal 1x1 RGBA PNG
        let mut encoder = png::Encoder::new(&mut output, 1, 1);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.add_text_chunk("chara".to_string(), encoded)?;

        let mut writer = encoder.write_header()?;
        writer.write_image_data(&[0, 0, 0, 0])?;
    }

    Ok(output)
}

/// Normalize card fields by parsing W++/structured content from `description`
/// into their proper V2 fields (personality, mes_example, scenario).
///
/// Many card authors stuff everything into `description` while leaving
/// `personality`, `mes_example`, and `scenario` empty. This function
/// detects and extracts:
///
/// - W++ attribute blocks like `[Name's Personality= "..."]` → `personality`
/// - W++ body blocks like `[Name's body= "..."]` → appended to `personality`
/// - Dialogue examples starting with `<START>` → `mes_example`
/// - Genre/Tags/Scenario metadata like `[Genre: ...; Scenario: ...]` → `scenario`
fn normalize_card_fields(card: &mut CharacterCardV2) {
    // Only redistribute if the target fields are empty
    if !card.personality.is_empty()
        && !card.mes_example.is_empty()
        && !card.scenario.is_empty()
    {
        return;
    }

    let desc = &card.description;
    let mut personality_parts: Vec<String> = Vec::new();
    let mut mes_example_parts: Vec<String> = Vec::new();
    let mut scenario_parts: Vec<String> = Vec::new();
    let mut remaining_desc = desc.clone();

    // Extract W++ attribute blocks: [Name's Key= "val1", "val2"]
    // Pattern: starts with '[', contains '=', ends with ']'
    let wpp_re = regex::Regex::new(r"(?s)\[[\w]+'s\s+(\w+)=\s*([^\]]*)\]").unwrap();
    for cap in wpp_re.captures_iter(desc) {
        let key = cap.get(1).unwrap().as_str().to_lowercase();
        let values_raw = cap.get(2).unwrap().as_str();
        let full_match = cap.get(0).unwrap().as_str();

        // Extract quoted values
        let quoted_re = regex::Regex::new(r#""([^"]*)""#).unwrap();
        let values: Vec<&str> = quoted_re
            .captures_iter(values_raw)
            .map(|c| c.get(1).unwrap().as_str())
            .collect();

        if !values.is_empty() {
            let label = if key == "personality" {
                "Personality".to_string()
            } else {
                // Capitalize first letter
                let mut chars = key.chars();
                match chars.next() {
                    None => key.clone(),
                    Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                }
            };
            personality_parts.push(format!("{}: {}", label, values.join(", ")));
        } else {
            // No quoted values — keep the raw block as personality
            personality_parts.push(full_match.to_string());
        }

        // Remove from description
        remaining_desc = remaining_desc.replace(full_match, "");
    }

    // Extract dialogue examples: anything starting with <START>
    if card.mes_example.is_empty() {
        if let Some(start_idx) = remaining_desc.find("<START>") {
            let dialogue = remaining_desc[start_idx..].trim().to_string();
            // Check if there's a genre/scenario block after the dialogue
            // Genre block pattern: [Genre: ...]
            if let Some(genre_idx) = dialogue.rfind('[') {
                let bracket_content = &dialogue[genre_idx..];
                if bracket_content.contains("Genre:")
                    || bracket_content.contains("Scenario:")
                    || bracket_content.contains("Tags:")
                {
                    // Extract scenario from the genre block if not already done
                    let meta_re =
                        regex::Regex::new(r"\[Genre:\s*([^;\]]+)(?:;\s*Tags:\s*([^;\]]+))?(?:;\s*Scenario:\s*([^\]]+))?\]")
                            .unwrap();
                    if card.scenario.is_empty() {
                        if let Some(cap) = meta_re.captures(bracket_content) {
                            if let Some(scenario) = cap.get(3) {
                                scenario_parts.push(scenario.as_str().trim().to_string());
                            } else if let Some(genre) = cap.get(1) {
                                scenario_parts.push(format!("Genre: {}", genre.as_str().trim()));
                            }
                        }
                    }
                    // The dialogue ends before the genre block
                    mes_example_parts.push(dialogue[..genre_idx].trim().to_string());
                } else {
                    mes_example_parts.push(dialogue.clone());
                }
            } else {
                mes_example_parts.push(dialogue.clone());
            }
            remaining_desc = remaining_desc[..start_idx].trim().to_string();
        }
    }

    // Extract genre/tags/scenario metadata that wasn't inside dialogue
    if card.scenario.is_empty() && scenario_parts.is_empty() {
        let meta_re =
            regex::Regex::new(r"\[Genre:\s*([^;\]]+)(?:;\s*Tags:\s*([^;\]]+))?(?:;\s*Scenario:\s*([^\]]+))?\]")
                .unwrap();
        if let Some(cap) = meta_re.captures(&remaining_desc) {
            let full_match = cap.get(0).unwrap().as_str();
            if let Some(scenario) = cap.get(3) {
                scenario_parts.push(scenario.as_str().trim().to_string());
            } else if let Some(genre) = cap.get(1) {
                scenario_parts.push(format!("Genre: {}", genre.as_str().trim()));
            }
            remaining_desc = remaining_desc.replace(full_match, "");
        }
    }

    // Clean up remaining description
    remaining_desc = remaining_desc
        .split('\n')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();

    // Apply extracted fields
    if !personality_parts.is_empty() && card.personality.is_empty() {
        card.personality = personality_parts.join("\n");
    }
    if !mes_example_parts.is_empty() && card.mes_example.is_empty() {
        card.mes_example = mes_example_parts.join("\n\n");
    }
    if !scenario_parts.is_empty() && card.scenario.is_empty() {
        card.scenario = scenario_parts.join("\n");
    }
    card.description = remaining_desc;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::character::CharacterCardV2;

    #[test]
    fn test_import_from_json_v2_spec_format() {
        // Test the V2 spec format with nested "data" key (as used in PNG tEXt chunks)
        let v2_spec_json = r#"{
            "spec": "chara_card_v2",
            "spec_version": "2.0",
            "data": {
                "name": "Seraphina",
                "description": "A magical healer",
                "personality": "Caring and protective",
                "scenario": "An enchanted forest",
                "first_mes": "Hello, traveler.",
                "mes_example": "<START>\n{{user}}: Hi\n{{char}}: Greetings!",
                "system_prompt": "You are Seraphina.",
                "post_history_instructions": "",
                "alternate_greetings": ["Welcome, friend."],
                "tags": ["fantasy", "magic"],
                "creator": "TestAuthor",
                "character_version": "1.0.0",
                "creator_notes": "A test character",
                "character_book": {},
                "extensions": {"talkativeness": 0.5}
            }
        }"#;

        let imported = import_from_json(v2_spec_json).expect("V2 spec import failed");
        assert_eq!(imported.name, "Seraphina");
        assert_eq!(imported.description, "A magical healer");
        assert_eq!(imported.personality, "Caring and protective");
        assert_eq!(imported.first_mes, "Hello, traveler.");
        assert_eq!(imported.tags, vec!["fantasy", "magic"]);
        assert_eq!(imported.creator, "TestAuthor");
        // ID should be a new UUID (not from the JSON)
        assert!(uuid::Uuid::parse_str(&imported.id).is_ok());
    }

    /// Helper to create a test CharacterCardV2 with predictable data.
    fn test_character() -> CharacterCardV2 {
        let mut c = CharacterCardV2::new("Test Character".to_string());
        c.description = "A test description".to_string();
        c.personality = "Bold and curious".to_string();
        c.scenario = "A test scenario".to_string();
        c.first_mes = "Hello, I am a test!".to_string();
        c.mes_example = "<START>\n{{user}}: Hi\n{{char}}: Hello!".to_string();
        c.tags = vec!["test".to_string(), "fixture".to_string()];
        c.creator = "TestSuite".to_string();
        c.alternate_greetings = vec!["Greeting 1".to_string(), "Greeting 2".to_string()];
        c.system_prompt = "You are a test character".to_string();
        c
    }

    #[test]
    fn test_export_to_json_roundtrip() {
        let character = test_character();
        let json = export_to_json(&character).expect("export failed");
        let parsed: CharacterCardV2 = serde_json::from_str(&json).expect("parse failed");

        // IDs will differ because export_to_json doesn't change the ID,
        // but we need to compare all other fields
        assert_eq!(character.name, parsed.name);
        assert_eq!(character.description, parsed.description);
        assert_eq!(character.personality, parsed.personality);
        assert_eq!(character.scenario, parsed.scenario);
        assert_eq!(character.first_mes, parsed.first_mes);
        assert_eq!(character.mes_example, parsed.mes_example);
        assert_eq!(character.tags, parsed.tags);
        assert_eq!(character.creator, parsed.creator);
        assert_eq!(character.alternate_greetings, parsed.alternate_greetings);
        assert_eq!(character.system_prompt, parsed.system_prompt);
        assert_eq!(character.id, parsed.id);
    }

    #[test]
    fn test_import_from_json_v2() {
        let character = test_character();
        let json = serde_json::to_string(&character).expect("serialize failed");
        let imported = import_from_json(&json).expect("import failed");

        assert_eq!(character.name, imported.name);
        assert_eq!(character.description, imported.description);
        assert_eq!(character.personality, imported.personality);
        assert_eq!(character.tags, imported.tags);
        // Note: ID will be different (new UUID assigned on import)
    }

    #[test]
    fn test_import_from_json_v1() {
        let v1_json = r#"{
            "char_name": "Alice",
            "char_persona": "A curious wanderer",
            "char_greeting": "Hello! I'm Alice.",
            "world_scenario": "A magical forest",
            "example_dialogue": "<START>\n{{user}}: Hi\n{{char}}: Hello!",
            "creator_notes": "A test character",
            "tags": "fantasy, adventure",
            "creator": "TestAuthor",
            "character_version": "1.0"
        }"#;

        let imported = import_from_json(v1_json).expect("V1 import failed");

        assert_eq!(imported.name, "Alice");
        assert_eq!(imported.personality, "A curious wanderer");
        assert_eq!(imported.first_mes, "Hello! I'm Alice.");
        assert_eq!(imported.scenario, "A magical forest");
        assert_eq!(imported.tags, vec!["fantasy", "adventure"]);
        assert_eq!(imported.creator, "TestAuthor");
        assert_eq!(imported.character_version, "2.0");
    }

    #[test]
    fn test_import_from_json_invalid() {
        let result = import_from_json("not valid json at all");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Invalid character card JSON"),
            "Expected descriptive error, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_export_to_png_with_text_chunk() {
        let character = test_character();
        let png_bytes = export_to_png(&character, None).expect("export to PNG failed");

        // Verify it's valid PNG by parsing it back
        let decoder = png::Decoder::new(Cursor::new(&png_bytes));
        let mut reader = decoder.read_info().expect("decode failed");
        let mut buf = vec![0u8; reader.output_buffer_size().unwrap()];
        reader.next_frame(&mut buf).expect("next_frame failed");
        reader.finish().expect("finish failed");

        let chara_chunks: Vec<_> = reader
            .info()
            .uncompressed_latin1_text
            .iter()
            .filter(|c| c.keyword == "chara")
            .collect();

        assert_eq!(chara_chunks.len(), 1, "Should have exactly one 'chara' tEXt chunk");

        // Verify the chunk contains valid base64 that decodes to CharacterCardV2 JSON
        let decoded = BASE64
            .decode(&chara_chunks[0].text)
            .expect("base64 decode failed");
        let json_str = String::from_utf8(decoded).expect("UTF-8 decode failed");
        let parsed: CharacterCardV2 = serde_json::from_str(&json_str).expect("JSON parse failed");
        assert_eq!(parsed.name, "Test Character");
    }

    #[test]
    fn test_import_from_png_extracts_chara() {
        // First create a PNG with embedded character data
        let character = test_character();
        let png_bytes = export_to_png(&character, None).expect("export failed");

        // Now import it back
        let (imported, avatar_data) =
            import_from_png(&png_bytes).expect("import from PNG failed");

        assert_eq!(imported.name, "Test Character");
        assert_eq!(imported.description, "A test description");
        assert_eq!(imported.tags, vec!["test", "fixture"]);
        assert!(avatar_data.is_some());
    }

    #[test]
    fn test_import_from_png_no_chara_chunk() {
        // Create a PNG without any tEXt chunk
        let mut output = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut output, 1, 1);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(&[0, 0, 0, 0]).unwrap();
        }

        let result = import_from_png(&output);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("No 'chara' tEXt chunk found"),
            "Expected no chara chunk error, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_png_roundtrip() {
        let character = test_character();
        let png_bytes = export_to_png(&character, None).expect("export failed");
        let (imported, _) = import_from_png(&png_bytes).expect("import failed");

        assert_eq!(character.name, imported.name);
        assert_eq!(character.description, imported.description);
        assert_eq!(character.personality, imported.personality);
        assert_eq!(character.scenario, imported.scenario);
        assert_eq!(character.first_mes, imported.first_mes);
        assert_eq!(character.mes_example, imported.mes_example);
        assert_eq!(character.tags, imported.tags);
        assert_eq!(character.creator, imported.creator);
        assert_eq!(character.alternate_greetings, imported.alternate_greetings);
        assert_eq!(character.system_prompt, imported.system_prompt);
        assert_eq!(character.post_history_instructions, imported.post_history_instructions);
        // ID will differ because import assigns new UUID
    }

    #[test]
    fn test_large_json_rejected() {
        // Create a JSON string that exceeds the 10MB limit
        let large_json = format!(
            r#"{{"id": "test", "name": "{}", "created_at": "2026-01-01T00:00:00", "updated_at": "2026-01-01T00:00:00"}}"#,
            "x".repeat(11 * 1024 * 1024) // >10MB name field
        );

        let result = import_from_json(&large_json);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("exceeds maximum size"),
            "Expected size limit error, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_normalize_wpp_description() {
        // Simulate a card where the author stuffed W++ personality, body,
        // dialogue examples, and scenario metadata all into `description`
        let v2_spec_json = r#"{
            "spec": "chara_card_v2",
            "spec_version": "2.0",
            "data": {
                "name": "Seraphina",
                "description": "[Seraphina's Personality= \"caring\", \"protective\", \"compassionate\"]\n[Seraphina's body= \"pink hair\", \"amber eyes\"]\n<START>\n{{user}}: Hi\n{{char}}: Greetings!\n[Genre: fantasy; Tags: adventure, Magic; Scenario: Attacked by beasts in the forest]",
                "personality": "",
                "scenario": "",
                "first_mes": "Hello traveler",
                "mes_example": "",
                "creator_notes": "",
                "system_prompt": "",
                "post_history_instructions": "",
                "tags": ["fantasy"],
                "creator": "TestAuthor",
                "character_version": "1.0.0",
                "alternate_greetings": [],
                "extensions": {},
                "character_book": {}
            }
        }"#;

        let card = import_from_json(v2_spec_json).expect("import failed");

        // Personality should contain extracted W++ traits
        assert!(
            card.personality.contains("Personality: caring, protective, compassionate"),
            "personality should have extracted personality traits, got: {}",
            card.personality
        );
        assert!(
            card.personality.contains("Body: pink hair, amber eyes"),
            "personality should have extracted body traits, got: {}",
            card.personality
        );

        // mes_example should contain the dialogue block
        assert!(
            card.mes_example.contains("<START>"),
            "mes_example should contain dialogue, got: {}",
            card.mes_example
        );
        assert!(
            card.mes_example.contains("{{user}}: Hi"),
            "mes_example should contain user dialogue, got: {}",
            card.mes_example
        );

        // scenario should contain the extracted scenario
        assert!(
            card.scenario.contains("Attacked by beasts in the forest"),
            "scenario should contain extracted scenario, got: {}",
            card.scenario
        );

        // description should now be clean (no W++ blocks, no dialogue, no genre metadata)
        assert!(
            !card.description.contains("[Seraphina's Personality"),
            "description should not contain W++ personality block, got: {}",
            card.description
        );
        assert!(
            !card.description.contains("<START>"),
            "description should not contain dialogue, got: {}",
            card.description
        );
        assert!(
            !card.description.contains("Genre:"),
            "description should not contain genre metadata, got: {}",
            card.description
        );
    }

    #[test]
    fn test_normalize_skips_when_fields_populated() {
        // If personality, mes_example, and scenario are already filled,
        // normalization should not modify description
        let v2_spec_json = r#"{
            "spec": "chara_card_v2",
            "spec_version": "2.0",
            "data": {
                "name": "Test",
                "description": "[Test's Personality= \"bold\"]\n<START>\n{{user}}: Hi\n[Genre: scifi; Scenario: Space travel]",
                "personality": "Already set",
                "scenario": "Already set",
                "first_mes": "Hi",
                "mes_example": "Already set",
                "creator_notes": "",
                "system_prompt": "",
                "post_history_instructions": "",
                "tags": [],
                "creator": "Test",
                "character_version": "1.0.0",
                "alternate_greetings": [],
                "extensions": {},
                "character_book": {}
            }
        }"#;

        let card = import_from_json(v2_spec_json).expect("import failed");

        // Nothing should have been extracted because all target fields are populated
        assert_eq!(card.personality, "Already set");
        assert_eq!(card.mes_example, "Already set");
        assert_eq!(card.scenario, "Already set");
    }
}
