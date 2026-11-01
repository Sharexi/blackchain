use crate::hash::{combine_hashes, Hash};

/// A Merkle tree node index and direction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofElement {
    Left(Hash),
    Right(Hash),
}

/// A cryptographic Merkle proof for verifying inclusion of a leaf in a block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerkleProof {
    pub leaf: Hash,
    pub path: Vec<ProofElement>,
}

impl MerkleProof {
    pub fn verify(&self, expected_root: &Hash) -> bool {
        let mut current = self.leaf;
        for step in &self.path {
            current = match step {
                ProofElement::Left(sibling) => combine_hashes(sibling, &current),
                ProofElement::Right(sibling) => combine_hashes(&current, sibling),
            };
        }
        current == *expected_root
    }
}

/// Compute the Merkle root of a list of leaf hashes.
pub fn calculate_merkle_root(leaves: &[Hash]) -> Hash {
    if leaves.is_empty() {
        return Hash::ZERO;
    }
    if leaves.len() == 1 {
        return leaves[0];
    }

    let mut current_level = leaves.to_vec();

    while current_level.len() > 1 {
        let mut next_level = Vec::with_capacity((current_level.len() + 1) / 2);

        for chunk in current_level.chunks(2) {
            if chunk.len() == 2 {
                next_level.push(combine_hashes(&chunk[0], &chunk[1]));
            } else {
                // Duplicate last element if odd
                next_level.push(combine_hashes(&chunk[0], &chunk[0]));
            }
        }
        current_level = next_level;
    }

    current_level[0]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::hash;

    #[test]
    fn test_empty_merkle_root() {
        assert_eq!(calculate_merkle_root(&[]), Hash::ZERO);
    }

    #[test]
    fn test_single_leaf() {
        let h = hash(b"single");
        assert_eq!(calculate_merkle_root(&[h]), h);
    }

    #[test]
    fn test_multiple_leaves() {
        let l1 = hash(b"leaf1");
        let l2 = hash(b"leaf2");
        let l3 = hash(b"leaf3");

        let root = calculate_merkle_root(&[l1, l2, l3]);
        assert_ne!(root, Hash::ZERO);
        assert_ne!(root, l1);
    }
}
