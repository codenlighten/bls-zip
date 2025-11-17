// Merkle Tree implementation for transaction verification
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

/// Merkle tree for efficient verification of transaction inclusion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleTree {
    /// The root hash of the tree
    pub root: [u8; 32],

    /// All leaf hashes
    leaves: Vec<[u8; 32]>,
}

impl MerkleTree {
    /// Build a new Merkle tree from a list of data
    pub fn new(data: Vec<Vec<u8>>) -> Self {
        let leaves: Vec<[u8; 32]> = data
            .iter()
            .map(|d| {
                let mut hasher = Sha3_256::new();
                hasher.update(d);
                hasher.finalize().into()
            })
            .collect();

        let root = Self::calculate_root(&leaves);

        Self { root, leaves }
    }

    /// Build from pre-computed hashes
    pub fn from_hashes(hashes: Vec<[u8; 32]>) -> Self {
        let root = Self::calculate_root(&hashes);
        Self {
            root,
            leaves: hashes,
        }
    }

    /// Calculate the Merkle root from leaf hashes
    fn calculate_root(leaves: &[[u8; 32]]) -> [u8; 32] {
        if leaves.is_empty() {
            return [0u8; 32];
        }

        let mut current_level = leaves.to_vec();

        while current_level.len() > 1 {
            if current_level.len() % 2 != 0 {
                current_level.push(*current_level.last().unwrap());
            }

            current_level = current_level
                .chunks(2)
                .map(|pair| {
                    let mut hasher = Sha3_256::new();
                    hasher.update(pair[0]);
                    hasher.update(pair[1]);
                    hasher.finalize().into()
                })
                .collect();
        }

        current_level[0]
    }

    /// Generate a Merkle proof for a specific leaf index
    pub fn generate_proof(&self, index: usize) -> Option<MerkleProof> {
        if index >= self.leaves.len() {
            return None;
        }

        let mut proof = Vec::new();
        let mut current_index = index;
        let mut current_level = self.leaves.clone();

        while current_level.len() > 1 {
            if current_level.len() % 2 != 0 {
                current_level.push(*current_level.last().unwrap());
            }

            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            if sibling_index < current_level.len() {
                proof.push(current_level[sibling_index]);
            }

            current_level = current_level
                .chunks(2)
                .map(|pair| {
                    let mut hasher = Sha3_256::new();
                    hasher.update(pair[0]);
                    hasher.update(pair[1]);
                    hasher.finalize().into()
                })
                .collect();

            current_index /= 2;
        }

        Some(MerkleProof {
            leaf: self.leaves[index],
            index,
            proof,
            root: self.root,
        })
    }

    /// Get the root hash
    pub fn root(&self) -> [u8; 32] {
        self.root
    }

    /// Get the number of leaves
    pub fn len(&self) -> usize {
        self.leaves.len()
    }

    /// Check if the tree is empty
    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }
}

/// Merkle proof for a specific leaf
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    /// The leaf being proven
    pub leaf: [u8; 32],

    /// Index of the leaf in the tree
    pub index: usize,

    /// Sibling hashes needed to recompute the root
    pub proof: Vec<[u8; 32]>,

    /// Expected root hash
    pub root: [u8; 32],
}

impl MerkleProof {
    /// Verify this Merkle proof
    pub fn verify(&self) -> bool {
        let mut current_hash = self.leaf;
        let mut current_index = self.index;

        for sibling in &self.proof {
            let mut hasher = Sha3_256::new();

            if current_index % 2 == 0 {
                hasher.update(current_hash);
                hasher.update(sibling);
            } else {
                hasher.update(sibling);
                hasher.update(current_hash);
            }

            current_hash = hasher.finalize().into();
            current_index /= 2;
        }

        current_hash == self.root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_tree_single_leaf() {
        let data = vec![b"transaction1".to_vec()];
        let tree = MerkleTree::new(data);

        assert_eq!(tree.len(), 1);
        assert!(!tree.is_empty());
    }

    #[test]
    fn test_merkle_tree_multiple_leaves() {
        let data = vec![
            b"tx1".to_vec(),
            b"tx2".to_vec(),
            b"tx3".to_vec(),
            b"tx4".to_vec(),
        ];
        let tree = MerkleTree::new(data);

        assert_eq!(tree.len(), 4);
    }

    #[test]
    fn test_merkle_proof() {
        let data = vec![
            b"tx1".to_vec(),
            b"tx2".to_vec(),
            b"tx3".to_vec(),
            b"tx4".to_vec(),
        ];
        let tree = MerkleTree::new(data);

        // Generate and verify proof for each leaf
        for i in 0..4 {
            let proof = tree.generate_proof(i).unwrap();
            assert!(proof.verify());
        }
    }

    #[test]
    fn test_empty_tree() {
        let tree = MerkleTree::new(vec![]);
        assert!(tree.is_empty());
        assert_eq!(tree.root(), [0u8; 32]);
    }
}
