//! Crypto utilities module
//! Provides hash, signature and other crypto-related functionality

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Calculate hash value of string
pub fn hash_string(input: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}

/// Calculate hash value of byte array
pub fn hash_bytes(input: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}

/// Simple checksum calculation
pub fn checksum(data: &[u8]) -> u32 {
    data.iter().fold(0u32, |acc, &byte| acc.wrapping_add(byte as u32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_string() {
        let hash1 = hash_string("test");
        let hash2 = hash_string("test");
        assert_eq!(hash1, hash2);
        
        let hash3 = hash_string("different");
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_hash_bytes() {
        let data = b"test data";
        let hash1 = hash_bytes(data);
        let hash2 = hash_bytes(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_checksum() {
        let data = b"test";
        let checksum = checksum(data);
        assert!(checksum > 0);
    }
}
