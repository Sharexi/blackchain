use std::fmt;
use std::str::FromStr;
use blackchain_crypto::{hash, PublicKey};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Native token amount in minimal indivisible units (1 Token = 10^18 units).
pub type Amount = u128;

/// Account transaction sequence counter.
pub type Nonce = u64;

/// Normalized compute unit (CU) representing standardized inference work.
pub type ComputeUnits = u64;

/// Blockchain account address, derived from the Ed25519 public key.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Address(pub [u8; 32]);

impl Address {
    pub const ZERO: Address = Address([0u8; 32]);

    pub fn from_public_key(pub_key: &PublicKey) -> Self {
        // Deterministically derive address by hashing the public key with Blake3
        let h = hash(pub_key.as_bytes());
        Address(*h.as_bytes())
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<PublicKey> for Address {
    fn from(pk: PublicKey) -> Self {
        Address::from_public_key(&pk)
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Address(0x{})", hex::encode(self.0))
    }
}

impl FromStr for Address {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let clean = s.strip_prefix("0x").unwrap_or(s);
        let mut bytes = [0u8; 32];
        hex::decode_to_slice(clean, &mut bytes)?;
        Ok(Address(bytes))
    }
}

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("0x{}", hex::encode(self.0)))
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Address::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blackchain_crypto::Keypair;

    #[test]
    fn test_address_derivation() {
        let keypair = Keypair::generate().expect("PQC keygen");
        let addr = Address::from_public_key(&keypair.public_key());
        assert_ne!(addr, Address::ZERO);
    }
}
