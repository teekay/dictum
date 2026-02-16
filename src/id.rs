use sha2::{Digest, Sha256};

/// Generate a hash-based ID: SHA256(title + created_at) -> base36, 6 chars.
/// The prefix is prepended with a hyphen separator.
pub fn generate_id(prefix: &str, title: &str, created_at: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(title.as_bytes());
    hasher.update(created_at.as_bytes());
    let hash = hasher.finalize();

    // Take first 4 bytes (32 bits) and convert to base36
    let num = u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]]);
    let b36 = base36_encode(num);

    // Pad to at least 3 chars, cap at 6
    let b36 = if b36.len() < 3 {
        format!("{:0>3}", b36)
    } else if b36.len() > 6 {
        b36[..6].to_string()
    } else {
        b36
    };

    format!("{}-{}", prefix, b36)
}

fn base36_encode(mut n: u32) -> String {
    if n == 0 {
        return "0".to_string();
    }
    const CHARS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut result = Vec::new();
    while n > 0 {
        result.push(CHARS[(n % 36) as usize]);
        n /= 36;
    }
    result.reverse();
    String::from_utf8(result).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id() {
        let id = generate_id("d", "We serve restaurant owners", "2024-01-15T10:30:00Z");
        assert!(id.starts_with("d-"));
        let hash_part = &id[2..];
        assert!(hash_part.len() >= 3 && hash_part.len() <= 6);
    }

    #[test]
    fn test_deterministic() {
        let id1 = generate_id("d", "test", "2024-01-01T00:00:00Z");
        let id2 = generate_id("d", "test", "2024-01-01T00:00:00Z");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_different_inputs() {
        let id1 = generate_id("d", "test1", "2024-01-01T00:00:00Z");
        let id2 = generate_id("d", "test2", "2024-01-01T00:00:00Z");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_base36_encode() {
        assert_eq!(base36_encode(0), "0");
        assert_eq!(base36_encode(36), "10");
        assert_eq!(base36_encode(35), "z");
    }
}
