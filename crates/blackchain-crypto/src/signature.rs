use std::str::FromStr;
use fips204::ml_dsa_44::{self, PK_LEN, SIG_LEN, SK_LEN};
use fips204::traits::{SerDes, Signer, Verifier};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

pub const CONTEXT_DOMAIN: &[u8] = b"blackchain-pqc-v1";

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Post-quantum signature verification failed")]
    InvalidSignature,
    #[error("Invalid post-quantum public key bytes (expected {expected}, got {got})")]
    InvalidPublicKeyLength { expected: usize, got: usize },
    #[error("Invalid post-quantum signature bytes (expected {expected}, got {got})")]
    InvalidSignatureLength { expected: usize, got: usize },
    #[error("FIPS-204 Key generation failed")]
    KeyGenError,
    #[error("FIPS-204 Signing failed")]
    SigningError,
    #[error("Hex decoding error: {0}")]
    HexError(#[from] hex::FromHexError),
}

/// NIST FIPS 204 ML-DSA-44 Post-Quantum Public Key (1312 bytes).
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PublicKey(pub Vec<u8>);

impl Default for PublicKey {
    fn default() -> Self {
        PublicKey(vec![0u8; PK_LEN])
    }
}

impl PublicKey {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() != PK_LEN {
            return Err(CryptoError::InvalidPublicKeyLength {
                expected: PK_LEN,
                got: bytes.len(),
            });
        }
        Ok(PublicKey(bytes.to_vec()))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Verify a post-quantum signature against a message using ML-DSA-44.
    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<(), CryptoError> {
        if self.0.len() != PK_LEN {
            return Err(CryptoError::InvalidPublicKeyLength {
                expected: PK_LEN,
                got: self.0.len(),
            });
        }
        if signature.0.len() != SIG_LEN {
            return Err(CryptoError::InvalidSignatureLength {
                expected: SIG_LEN,
                got: signature.0.len(),
            });
        }

        let mut pk_arr = [0u8; PK_LEN];
        pk_arr.copy_from_slice(&self.0);

        let fips_pk = ml_dsa_44::PublicKey::try_from_bytes(pk_arr)
            .map_err(|_| CryptoError::InvalidSignature)?;

        let mut sig_arr = [0u8; SIG_LEN];
        sig_arr.copy_from_slice(&signature.0);

        if fips_pk.verify(message, &sig_arr, CONTEXT_DOMAIN) {
            Ok(())
        } else {
            Err(CryptoError::InvalidSignature)
        }
    }
}

impl std::fmt::Display for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}

impl std::fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PQPublicKey(0x{}...{})",
            hex::encode(&self.0[..8.min(self.0.len())]),
            hex::encode(&self.0[self.0.len().saturating_sub(8)..])
        )
    }
}

impl FromStr for PublicKey {
    type Err = CryptoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let clean = s.strip_prefix("0x").unwrap_or(s);
        let bytes = hex::decode(clean)?;
        PublicKey::from_bytes(&bytes)
    }
}

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("0x{}", hex::encode(&self.0)))
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        PublicKey::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// NIST FIPS 204 ML-DSA-44 Post-Quantum Signature (2420 bytes).
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Signature(pub Vec<u8>);

impl Default for Signature {
    fn default() -> Self {
        Signature(vec![0u8; SIG_LEN])
    }
}

impl Signature {
    pub const ZERO: Signature = Signature(Vec::new());

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() != SIG_LEN {
            return Err(CryptoError::InvalidSignatureLength {
                expected: SIG_LEN,
                got: bytes.len(),
            });
        }
        Ok(Signature(bytes.to_vec()))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl std::fmt::Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}

impl std::fmt::Debug for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            write!(f, "PQSignature(EMPTY)")
        } else {
            write!(
                f,
                "PQSignature(0x{}...{})",
                hex::encode(&self.0[..8.min(self.0.len())]),
                hex::encode(&self.0[self.0.len().saturating_sub(8)..])
            )
        }
    }
}

impl FromStr for Signature {
    type Err = CryptoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let clean = s.strip_prefix("0x").unwrap_or(s);
        let bytes = hex::decode(clean)?;
        if bytes.is_empty() {
            return Ok(Signature(Vec::new()));
        }
        Signature::from_bytes(&bytes)
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("0x{}", hex::encode(&self.0)))
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Signature::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// NIST FIPS 204 ML-DSA-44 Post-Quantum Keypair.
pub struct Keypair {
    private_key: ml_dsa_44::PrivateKey,
    public_key: PublicKey,
}

impl Keypair {
    /// Generate a fresh post-quantum ML-DSA-44 keypair.
    pub fn generate() -> Result<Self, CryptoError> {
        let (pk, sk) = ml_dsa_44::try_keygen().map_err(|_| CryptoError::KeyGenError)?;
        let pk_bytes = pk.into_bytes().to_vec();
        Ok(Keypair {
            private_key: sk,
            public_key: PublicKey(pk_bytes),
        })
    }

    pub fn public_key(&self) -> PublicKey {
        self.public_key.clone()
    }

    /// Sign a message using ML-DSA-44 with domain context separation.
    pub fn sign(&self, message: &[u8]) -> Result<Signature, CryptoError> {
        let sig_arr = self
            .private_key
            .try_sign(message, CONTEXT_DOMAIN)
            .map_err(|_| CryptoError::SigningError)?;
        Ok(Signature(sig_arr.to_vec()))
    }

    /// Export private key bytes for storage.
    pub fn secret_bytes(&self) -> [u8; SK_LEN] {
        self.private_key.clone().into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_quantum_sign_and_verify() {
        let keypair = Keypair::generate().expect("ML-DSA-44 keygen must succeed");
        let pub_key = keypair.public_key();
        let message = b"Blackchain Post-Quantum Compute Verification";

        // Sign with ML-DSA-44
        let sig = keypair.sign(message).expect("Signing must succeed");
        assert_eq!(sig.as_bytes().len(), SIG_LEN);
        assert_eq!(pub_key.as_bytes().len(), PK_LEN);

        // Verify with ML-DSA-44
        assert!(pub_key.verify(message, &sig).is_ok());

        // Tamper test
        let tampered_msg = b"Tampered compute output";
        assert!(pub_key.verify(tampered_msg, &sig).is_err());
    }
}
