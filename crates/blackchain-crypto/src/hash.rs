use core::fmt;
use std::str::FromStr;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A 32-byte cryptographic hash (Blake3).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    pub const ZERO: Hash = Hash([0u8; 32]);

    pub fn new(bytes: [u8; 32]) -> Self {
        Hash(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash(0x{})", hex::encode(self.0))
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl FromStr for Hash {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let clean = s.strip_prefix("0x").unwrap_or(s);
        let mut bytes = [0u8; 32];
        hex::decode_to_slice(clean, &mut bytes)?;
        Ok(Hash(bytes))
    }
}

impl Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(self.0))
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Hash::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// Compute a Blake3 hash of the provided bytes.
pub fn hash(data: &[u8]) -> Hash {
    let output = blake3::hash(data);
    Hash(*output.as_bytes())
}

/// Combine and hash two hashes (Merkle node computation).
pub fn combine_hashes(left: &Hash, right: &Hash) -> Hash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(left.as_bytes());
    hasher.update(right.as_bytes());
    Hash(*hasher.finalize().as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_computation() {
        let h1 = hash(b"hello blackchain");
        let h2 = hash(b"hello blackchain");
        let h3 = hash(b"different input");
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_hash_hex_serde() {
        let h = hash(b"test");
        let serialized = serde_json::to_string(&h).unwrap();
        let deserialized: Hash = serde_json::from_str(&serialized).unwrap();
        assert_eq!(h, deserialized);
    }
}
