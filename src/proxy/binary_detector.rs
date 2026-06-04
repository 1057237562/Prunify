/// Detects binary (non-text) data by inspecting byte patterns.
pub struct BinaryDetector;

impl BinaryDetector {
    /// Check if data appears to be binary (non-text).
    /// Returns true if data contains null bytes or >30% non-printable characters
    /// in the first 8KB.
    pub fn is_binary(data: &[u8]) -> bool {
        let check_len = data.len().min(8192);
        let slice = &data[..check_len];

        // Null byte check
        if slice.contains(&0u8) {
            return true;
        }

        // Non-printable character ratio
        let total = slice.len();
        if total == 0 {
            return false;
        }
        let non_printable = slice
            .iter()
            .filter(|&&b| b != b'\n' && b != b'\r' && b != b'\t' && !(32..=126).contains(&b))
            .count();

        (non_printable as f64 / total as f64) > 0.30
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_data_not_binary() {
        assert!(!BinaryDetector::is_binary(b""));
    }

    #[test]
    fn test_single_null_byte() {
        assert!(BinaryDetector::is_binary(b"\x00"));
    }

    #[test]
    fn test_just_below_threshold() {
        // 30% of 100 = 30 non-printable → 29 should pass as text
        let mut data = vec![b'a'; 100];
        // Replace 29 bytes with non-printable (but not null)
        for i in 0..29 {
            data[i] = 0x01;
        }
        assert!(!BinaryDetector::is_binary(&data));
    }

    #[test]
    fn test_just_above_threshold() {
        // 31 non-printable out of 100 → binary
        let mut data = vec![b'a'; 100];
        for i in 0..31 {
            data[i] = 0x01;
        }
        assert!(BinaryDetector::is_binary(&data));
    }

    #[test]
    fn test_newlines_tabs_not_counted_as_binary() {
        let data = b"\n\n\n\n\n\n\n\n\n\n\t\t\t\t\t\t\r\r\r\r\r";
        // All are allowed control chars, so non_printable = 0
        assert!(!BinaryDetector::is_binary(data));
    }

    #[test]
    fn test_truncated_to_8kb() {
        // Create 16KB of text with null bytes only after 8KB
        let mut data = vec![b'a'; 16384];
        data[8192] = 0x00; // null byte at 8KB boundary
        // Since we only check first 8KB, this should be false
        // Actually wait — index 8192 is at position 8193 which is past the first 8KB (slice is 0..8192)
        // So the null byte won't be seen — should be false
        assert!(!BinaryDetector::is_binary(&data));
    }

    #[test]
    fn test_binary_in_first_8kb() {
        // Create 16KB of text with null byte at position 4096
        let mut data = vec![b'a'; 16384];
        data[4096] = 0x00;
        // Null byte is within first 8KB → binary
        assert!(BinaryDetector::is_binary(&data));
    }
}
