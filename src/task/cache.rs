use macros_rs::crashln;
use merkle_hash::{bytes_to_hex, Algorithm, MerkleTree};

pub fn create_hash(path: &str) -> String {
    let tree = match MerkleTree::builder(path).algorithm(Algorithm::Blake3).hash_names(false).build() {
        Ok(v) => v,
        Err(e) => crashln!("Invalid UTF-8 sequence: {}", e),
    };

    bytes_to_hex(tree.root.item.hash)
}
