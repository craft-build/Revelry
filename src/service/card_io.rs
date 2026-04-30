use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::io::Cursor;

use crate::domain::character::{CharacterCardV2, CharacterV1};

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

    // Try V2 format first
    if let Ok(mut card) = serde_json::from_str::<CharacterCardV2>(json_str) {
        // Assign a new UUID for imported characters
        card.id = uuid::Uuid::new_v4().to_string();
        return Ok(card);
    }

    // Try V1 format
    if let Ok(v1) = serde_json::from_str::<CharacterV1>(json_str) {
        return Ok(v1.to_v2());
    }

    Err(anyhow!(
        "Invalid character card JSON. Expected CharacterCardV2 or CharacterV1 format."
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
    let reader = decoder.read_info()?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::character::CharacterCardV2;

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
        let reader = decoder.read_info().expect("decode failed");

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
}
