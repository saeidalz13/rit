use sha2::{Digest, Sha256};

pub fn get_hash_from_file(content: &Vec<u8>) -> (String, Vec<u8>) {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let hash_result = hasher.finalize();
    (format!("{:x}", hash_result), hash_result.to_vec())
}
