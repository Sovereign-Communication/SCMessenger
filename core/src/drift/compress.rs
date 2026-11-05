/// LZ4 compression wrapper for Drift Protocol payloads
use super::DriftError;

/// Compress data using LZ4 with size prepend
///
/// The lz4_flex::compress_prepend_size function automatically adds
/// the uncompressed size at the beginning, allowing automatic decompression.
pub fn compress(data: &[u8]) -> Vec<u8> {
    lz4_flex::compress_prepend_size(data)
}

/// Decompress data that was compressed with `compress()`
///
/// Returns error if decompression fails.
pub fn decompress(data: &[u8]) -> Result<Vec<u8>, DriftError> {
    if data.len() >= 4 {
        let declared_size = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        if declared_size > crate::message::codec::MAX_MESSAGE_SIZE {
            return Err(DriftError::DecompressionFailed(
                "Decompressed ciphertext exceeds the maximum message size".to_string(),
            ));
        }
    }

    lz4_flex::decompress_size_prepended(data)
        .map_err(|e| DriftError::DecompressionFailed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress_roundtrip() {
        let original = b"Hello, Drift Protocol! This is a test message.";
        let compressed = compress(original);
        let decompressed = decompress(&compressed).unwrap();

        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_compress_empty_data() {
        let original = b"";
        let compressed = compress(original);
        let decompressed = decompress(&compressed).unwrap();

        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_compress_single_byte() {
        let original = b"A";
        let compressed = compress(original);
        let decompressed = decompress(&compressed).unwrap();

        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_compress_large_data() {
        let original = vec![0x42u8; 100_000];
        let compressed = compress(&original);
        let decompressed = decompress(&compressed).unwrap();

        assert_eq!(decompressed.len(), original.len());
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_decompress_invalid_data() {
        let result = decompress(b"not compressed data");
        assert!(result.is_err());
    }

    #[test]
    fn test_compress_repetitive_data() {
        // Repetitive data should compress well
        let original = "AAAAAABBBBBBCCCCCCDDDDDD".repeat(100);
        let compressed = compress(original.as_bytes());

        // Compressed should be significantly smaller
        assert!(compressed.len() < original.len() / 2);

        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(decompressed, original.as_bytes());
    }

    #[test]
    fn test_compress_random_data() {
        // Random data may not compress well
        let original: Vec<u8> = (0..1000)
            .map(|i| (i % 256) as u8)
            .cycle()
            .take(1000)
            .collect();
        let compressed = compress(&original);
        let decompressed = decompress(&compressed).unwrap();

        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_compress_text_data() {
        let original = "The quick brown fox jumps over the lazy dog. \
                        The quick brown fox jumps over the lazy dog. \
                        The quick brown fox jumps over the lazy dog."
            .as_bytes();

        let compressed = compress(original);
        let decompressed = decompress(&compressed).unwrap();

        assert_eq!(decompressed, original);
    }
}
