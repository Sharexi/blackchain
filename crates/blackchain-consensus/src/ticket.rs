use blackchain_crypto::{hash, Hash};

/// Compute the PoUW lottery ticket from an inference execution receipt and previous block hash.
pub fn compute_pouw_ticket(
    receipt_hash: &Hash,
    prev_block_hash: &Hash,
    nonce: u64,
) -> Hash {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(receipt_hash.as_bytes());
    bytes.extend_from_slice(prev_block_hash.as_bytes());
    bytes.extend_from_slice(&nonce.to_be_bytes());
    hash(&bytes)
}

/// Generate a deterministic synthetic calibration prompt when the user inference queue is empty.
pub fn generate_synthetic_prompt(prev_block_hash: &Hash, height: u64) -> String {
    format!(
        "Blackchain PoUW Benchmark Task [Height: {}, Seed: {}]: Summarize the principle of verifiable AI compute in 50 words.",
        height,
        hex::encode(&prev_block_hash.as_bytes()[0..8])
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticket_determinism() {
        let r_hash = hash(b"receipt1");
        let prev_hash = hash(b"prev_block");
        let nonce = 42;

        let t1 = compute_pouw_ticket(&r_hash, &prev_hash, nonce);
        let t2 = compute_pouw_ticket(&r_hash, &prev_hash, nonce);
        assert_eq!(t1, t2);

        let t3 = compute_pouw_ticket(&r_hash, &prev_hash, nonce + 1);
        assert_ne!(t1, t3);
    }
}
