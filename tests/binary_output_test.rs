#[test]
fn test_random_binary_data_detected_as_binary() {
    // Generate 1000 bytes of binary data (non-printable, no null bytes to test ratio)
    let binary_data: Vec<u8> = (0..1000)
        .map(|i| ((i % 200) as u8).wrapping_add(0x80))
        .collect();
    assert!(
        prunify::proxy::binary_detector::BinaryDetector::is_binary(&binary_data),
        "Binary data with >30% non-printable chars should be detected as binary"
    );
}

#[test]
fn test_valid_text_not_detected_as_binary() {
    let text = b"Hello, world! This is a perfectly normal text string with no binary content.
It has multiple lines and various punctuation: !@#$%^&*()_+-=[]{}|;':\",./<>?~
The quick brown fox jumps over the lazy dog.
1234567890 and some more text to ensure we have enough data for the check.
This should definitely NOT be detected as binary since all characters are printable ASCII.
Let's add a few more lines to be thorough and exceed any minimum length requirements.
Typically binary detectors look at the first 8KB of data, so we need substantial text.
Padding padding padding padding padding padding padding padding padding padding.
More text here to ensure we have a decent sample size for the ratio calculation.
Alright, this should be more than enough printable text to pass the test.";

    assert!(
        !prunify::proxy::binary_detector::BinaryDetector::is_binary(text),
        "Valid printable ASCII text should NOT be detected as binary"
    );
}

#[test]
fn test_null_bytes_detected_as_binary() {
    // Text with embedded null bytes (e.g., UTF-16, or corrupted text)
    let mut data: Vec<u8> = b"Hello, this is text with ".to_vec();
    data.push(0u8); // null byte
    data.extend_from_slice(b"a null byte in the middle");
    data.push(0u8); // another null byte
    data.extend_from_slice(b" and more text after");

    assert!(
        prunify::proxy::binary_detector::BinaryDetector::is_binary(&data),
        "Data containing null bytes should be detected as binary regardless of printable content"
    );
}
