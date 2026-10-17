//! Shared helpers for the embedding subsystem: cosine similarity, a stable
//! content hash, and f32 vector ↔ BLOB encoding for SQLite storage.

/// Cosine similarity between two vectors. Returns 0.0 for empty or
/// length-mismatched inputs (treated as "no information", never NaN).
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;
    for i in 0..a.len() {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom <= f32::EPSILON {
        return 0.0;
    }
    dot / denom
}

/// Stable FNV-1a 64-bit hash, rendered as hex. Used as the `content_hash` of
/// embedding rows so unchanged content is never re-embedded and stale vectors
/// (edited messages/entries) can be skipped on read. Deterministic across
/// process restarts, unlike std's randomized hashers.
pub fn content_hash(text: &str) -> String {
    format!("{:016x}", fnv1a64(text.as_bytes()))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Encodes a vector as little-endian f32 bytes for BLOB storage.
pub fn encode_vector(vector: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vector.len() * 4);
    for v in vector {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    bytes
}

/// Decodes little-endian f32 bytes into a vector. Returns an empty vec for
/// malformed input (length not a multiple of 4).
pub fn decode_vector(bytes: &[u8]) -> Vec<f32> {
    let (chunks, remainder) = bytes.as_chunks::<4>();
    if !remainder.is_empty() {
        return Vec::new();
    }
    chunks.iter().map(|c| f32::from_le_bytes(*c)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_identical_vectors() {
        let v = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&v, &v);
        assert!(
            (sim - 1.0).abs() < 1e-6,
            "identical vectors → 1.0, got {sim}"
        );
    }

    #[test]
    fn test_cosine_orthogonal_vectors() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!(cosine_similarity(&a, &b).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_opposite_vectors() {
        let a = vec![1.0, 0.5];
        let b = vec![-1.0, -0.5];
        assert!((cosine_similarity(&a, &b) + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_known_value() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 1.0, 0.0];
        let expected = 1.0 / 2.0f32.sqrt();
        assert!((cosine_similarity(&a, &b) - expected).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_mismatched_lengths() {
        assert_eq!(cosine_similarity(&[1.0], &[1.0, 2.0]), 0.0);
        assert_eq!(cosine_similarity(&[], &[]), 0.0);
    }

    #[test]
    fn test_cosine_zero_vector() {
        assert_eq!(cosine_similarity(&[0.0, 0.0], &[1.0, 2.0]), 0.0);
    }

    #[test]
    fn test_content_hash_stable() {
        assert_eq!(content_hash("dragon lore"), content_hash("dragon lore"));
        assert_ne!(content_hash("dragon lore"), content_hash("dragon lore "));
    }

    #[test]
    fn test_content_hash_deterministic_across_calls() {
        // FNV-1a must not depend on process-random state.
        let h1 = content_hash("some message content");
        let h2 = content_hash("some message content");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 16);
    }

    #[test]
    fn test_vector_roundtrip() {
        let v = vec![0.1, -0.5, 3.25, 0.0, 1e-7];
        let encoded = encode_vector(&v);
        assert_eq!(encoded.len(), v.len() * 4);
        let decoded = decode_vector(&encoded);
        assert_eq!(decoded.len(), v.len());
        for (a, b) in v.iter().zip(decoded.iter()) {
            assert!((a - b).abs() < 1e-9);
        }
    }

    #[test]
    fn test_decode_malformed() {
        assert!(decode_vector(&[0, 0, 0]).is_empty());
        assert!(decode_vector(&[]).is_empty());
    }
}
