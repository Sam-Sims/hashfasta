use std::hash::Hasher;

use hex::encode;
use highway::{HighwayHash, HighwayHasher, Key};
use md5::{Digest as Md5Digest, Md5};
use sha2::{Digest as Sha2Digest, Sha256};

pub enum HashAlgorithm {
    Highway,
    Md5,
    Sha2,
}

pub fn calculate_hash(algorithm: &HashAlgorithm, sequence: &[u8]) -> String {
    match algorithm {
        HashAlgorithm::Highway => HighwayHasher::default().hash64(sequence).to_string(),
        HashAlgorithm::Md5 => {
            let hash = Md5::digest(sequence);
            encode(hash)
        }
        HashAlgorithm::Sha2 => {
            let hash = Sha256::digest(sequence);
            encode(hash)
        }
    }
}

pub fn calculate_final_hash(algorithm: &HashAlgorithm, hashes: &[String]) -> String {
    match algorithm {
        HashAlgorithm::Highway => {
            let key = Key([1, 2, 3, 4]);
            let mut hasher = HighwayHasher::new(key);
            for hash in hashes {
                hasher.write_u64(hash.parse().unwrap());
            }
            hasher.finalize64().to_string()
        }
        HashAlgorithm::Md5 => {
            let mut hasher = Md5::new();
            for hash in hashes {
                hasher.update(hash);
            }
            encode(hasher.finalize())
        }
        HashAlgorithm::Sha2 => {
            let mut hasher = Sha256::new();
            for hash in hashes {
                hasher.update(hash);
            }
            encode(hasher.finalize())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{lookup, rc_lookup};

    use super::*;

    #[test]
    fn test_lookup() {
        assert_eq!(lookup(b'A'), 1);
        assert_eq!(lookup(b'C'), 2);
        assert_eq!(lookup(b'G'), 3);
        assert_eq!(lookup(b'T'), 4);
    }

    #[test]
    fn test_rc_lookup() {
        assert_eq!(rc_lookup(b'A'), 4);
        assert_eq!(rc_lookup(b'C'), 3);
        assert_eq!(rc_lookup(b'G'), 2);
        assert_eq!(rc_lookup(b'T'), 1);
    }

    #[test]
    fn test_highway_same() {
        let sequence = b"ATCG";
        let sequence2 = b"ATCG";
        let highway_hash = calculate_hash(&HashAlgorithm::Highway, sequence);
        let highway_hash2 = calculate_hash(&HashAlgorithm::Highway, sequence2);
        assert_eq!(highway_hash, highway_hash2);
    }

    #[test]
    fn test_md5_same() {
        let sequence = b"ATCG";
        let sequence2 = b"ATCG";
        let md5_hash = calculate_hash(&HashAlgorithm::Md5, sequence);
        let md5_hash2 = calculate_hash(&HashAlgorithm::Md5, sequence2);
        assert_eq!(md5_hash, md5_hash2);
    }

    #[test]
    fn test_sha2_same() {
        let sequence = b"ATCG";
        let sequence2 = b"ATCG";
        let sha2_hash = calculate_hash(&HashAlgorithm::Sha2, sequence);
        let sha2_hash2 = calculate_hash(&HashAlgorithm::Sha2, sequence2);
        assert_eq!(sha2_hash, sha2_hash2);
    }

    #[test]
    fn test_highway_different() {
        let sequence = b"ATCG";
        let sequence2 = b"CGAT";
        let highway_hash = calculate_hash(&HashAlgorithm::Highway, sequence);
        let highway_hash2 = calculate_hash(&HashAlgorithm::Highway, sequence2);
        assert_ne!(highway_hash, highway_hash2);
    }

    #[test]
    fn test_md5_different() {
        let sequence = b"ATCG";
        let sequence2 = b"CGAT";
        let md5_hash = calculate_hash(&HashAlgorithm::Md5, sequence);
        let md5_hash2 = calculate_hash(&HashAlgorithm::Md5, sequence2);
        assert_ne!(md5_hash, md5_hash2);
    }

    #[test]
    fn test_sha2_different() {
        let sequence = b"ATCG";
        let sequence2 = b"CGAT";
        let sha2_hash = calculate_hash(&HashAlgorithm::Sha2, sequence);
        let sha2_hash2 = calculate_hash(&HashAlgorithm::Sha2, sequence2);
        assert_ne!(sha2_hash, sha2_hash2);
    }
}
