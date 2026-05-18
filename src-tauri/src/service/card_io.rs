use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::io::Cursor;

use crate::domain::character::{CharacterCardV2, CharaCardV2Spec, CharacterV1};

/// Maximum allowed JSON payload size (10 MB).
const MAX_JSON_SIZE: usize = 10 * 1024 * 1024;

/// Import a character card from a JSON string.
///
/// Attempts V2 spec format first, then V1 with auto-migration.
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
    let mut img_data = vec![0u8; reader
        .output_buffer_size()
        .ok_or_else(|| anyhow!("Cannot determine PNG output buffer size"))?];
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
