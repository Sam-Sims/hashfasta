use serde::ser::{Serialize, Serializer};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::hash::Hasher;
use twox_hash::XxHash3_64;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct SequenceHash(pub [u8; 8]);

const NORMALISE_LUT: [u8; 256] = lookup_table();
const COMPLEMENT_LUT: [u8; 256] = rc_lookup_table();

const fn lookup_table() -> [u8; 256] {
    let mut table = [b'N'; 256];
    table[b'A' as usize] = b'A';
    table[b'a' as usize] = b'A';
    table[b'C' as usize] = b'C';
    table[b'c' as usize] = b'C';
    table[b'G' as usize] = b'G';
    table[b'g' as usize] = b'G';
    table[b'T' as usize] = b'T';
    table[b't' as usize] = b'T';
    table[b'U' as usize] = b'T';
    table[b'u' as usize] = b'T';
    table[b'-' as usize] = b'-';
    table
}

#[allow(clippy::cast_possible_truncation)]
const fn rc_lookup_table() -> [u8; 256] {
    let mut table = [0u8; 256];
    let mut i = 0usize;
    while i < 256 {
        table[i] = i as u8;
        i += 1;
    }
    table[b'A' as usize] = b'T';
    table[b'a' as usize] = b't';
    table[b'C' as usize] = b'G';
    table[b'c' as usize] = b'g';
    table[b'G' as usize] = b'C';
    table[b'g' as usize] = b'c';
    table[b'T' as usize] = b'A';
    table[b't' as usize] = b'a';
    table[b'U' as usize] = b'A';
    table[b'u' as usize] = b'a';
    table
}

fn normalise_sequence(seq: &[u8]) -> Vec<u8> {
    let mut normalised = Vec::with_capacity(seq.len());
    for &b in seq {
        normalised.push(NORMALISE_LUT[b as usize]);
    }
    normalised
}

fn reverse_complement(seq: &[u8]) -> Vec<u8> {
    let mut reverse_complement = Vec::with_capacity(seq.len());
    for &b in seq.iter().rev() {
        reverse_complement.push(COMPLEMENT_LUT[b as usize]);
    }
    reverse_complement
}

fn canonicalise_sequence(seq: &[u8], normalise: bool) -> Vec<u8> {
    let forward = if normalise {
        normalise_sequence(seq)
    } else {
        seq.to_vec()
    };
    let reverse = reverse_complement(&forward);
    if forward <= reverse {
        forward
    } else {
        reverse
    }
}

pub fn hash_to_string(hash: SequenceHash) -> String {
    hex::encode(hash.0)
}

impl Serialize for SequenceHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex = hash_to_string(*self);
        serializer.serialize_str(&hex)
    }
}

pub fn calculate_final_hash(hashes: &[SequenceHash]) -> String {
    let mut sorted_hashes = hashes.to_vec();
    sorted_hashes.sort();
    let mut hash_writer = XxHash3_64::new();
    for hash in &sorted_hashes {
        hash_writer.write(&hash.0);
    }
    let final_hash = SequenceHash(hash_writer.finish().to_be_bytes());
    hash_to_string(final_hash)
}

pub fn count_duplicates(
    hashes: &[SequenceHash],
) -> HashMap<SequenceHash, usize, BuildHasherDefault<XxHash3_64>> {
    let mut counts = HashMap::with_capacity_and_hasher(
        hashes.len(),
        BuildHasherDefault::<XxHash3_64>::default(),
    );
    for hash in hashes {
        *counts.entry(*hash).or_insert(0) += 1;
    }
    counts
}

pub fn hash_sequence_bytes(seq: &[u8], normalise: bool, canonicalise: bool) -> SequenceHash {
    if canonicalise {
        let canonical = canonicalise_sequence(seq, normalise);
        SequenceHash(XxHash3_64::oneshot(&canonical).to_be_bytes())
    } else if normalise {
        let normalised = normalise_sequence(seq);
        SequenceHash(XxHash3_64::oneshot(&normalised).to_be_bytes())
    } else {
        SequenceHash(XxHash3_64::oneshot(seq).to_be_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalise_lut_valid_bases() {
        assert_eq!(NORMALISE_LUT[b'A' as usize], b'A');
        assert_eq!(NORMALISE_LUT[b'a' as usize], b'A');
        assert_eq!(NORMALISE_LUT[b'C' as usize], b'C');
        assert_eq!(NORMALISE_LUT[b'c' as usize], b'C');
        assert_eq!(NORMALISE_LUT[b'G' as usize], b'G');
        assert_eq!(NORMALISE_LUT[b'g' as usize], b'G');
        assert_eq!(NORMALISE_LUT[b'T' as usize], b'T');
        assert_eq!(NORMALISE_LUT[b't' as usize], b'T');
        assert_eq!(NORMALISE_LUT[b'U' as usize], b'T');
        assert_eq!(NORMALISE_LUT[b'u' as usize], b'T');
        assert_eq!(NORMALISE_LUT[b'-' as usize], b'-');
    }

    #[test]
    fn test_complement_lut_valid_bases() {
        assert_eq!(COMPLEMENT_LUT[b'A' as usize], b'T');
        assert_eq!(COMPLEMENT_LUT[b'a' as usize], b't');
        assert_eq!(COMPLEMENT_LUT[b'C' as usize], b'G');
        assert_eq!(COMPLEMENT_LUT[b'c' as usize], b'g');
        assert_eq!(COMPLEMENT_LUT[b'G' as usize], b'C');
        assert_eq!(COMPLEMENT_LUT[b'g' as usize], b'c');
        assert_eq!(COMPLEMENT_LUT[b'T' as usize], b'A');
        assert_eq!(COMPLEMENT_LUT[b't' as usize], b'a');
        assert_eq!(COMPLEMENT_LUT[b'U' as usize], b'A');
        assert_eq!(COMPLEMENT_LUT[b'u' as usize], b'a');
        assert_eq!(COMPLEMENT_LUT[b'-' as usize], b'-');
        assert_eq!(COMPLEMENT_LUT[b'N' as usize], b'N');
        assert_eq!(COMPLEMENT_LUT[b'?' as usize], b'?');
    }

    #[test]
    fn test_normalise_sequence() {
        let input = b"acgtuN-";
        let expected = b"ACGTTN-".to_vec();
        assert_eq!(normalise_sequence(input), expected);
    }

    #[test]
    fn test_reverse_complement() {
        let input = b"ACGTu";
        let expected = b"aACGT".to_vec();
        assert_eq!(reverse_complement(input), expected);
    }

    #[test]
    fn test_canonicalise_sequence() {
        let input = b"ACGTGGA";
        let expected = b"ACGTGGA".to_vec();
        assert_eq!(canonicalise_sequence(input, false), expected);
    }

    #[test]
    fn test_canonicalise_sequence_actually_do_it() {
        let input = b"GTCGAT";
        let expected = b"ATCGAC".to_vec();
        assert_eq!(canonicalise_sequence(input, false), expected);
    }

    #[test]
    fn test_canonicalise_sequence_normalised() {
        let input = b"acgtu";
        let expected = b"AACGT".to_vec();
        assert_eq!(canonicalise_sequence(input, true), expected);
    }

    #[test]
    fn test_hash_sequence_valid() {
        let input = b"acgtu";
        let canonical = canonicalise_sequence(input, true);
        let expected = SequenceHash(XxHash3_64::oneshot(&canonical).to_be_bytes());
        assert_eq!(hash_sequence_bytes(input, true, true), expected);
    }

    #[test]
    fn test_count_duplicates_valid() {
        let a = SequenceHash([0u8; 8]);
        let b = SequenceHash([1u8; 8]);
        let c = SequenceHash([2u8; 8]);
        let hashes = vec![a, b, a, c, a, b];
        let counts = count_duplicates(&hashes);

        assert_eq!(counts.len(), 3);
        assert_eq!(counts.get(&a), Some(&3));
        assert_eq!(counts.get(&b), Some(&2));
        assert_eq!(counts.get(&c), Some(&1));
    }
}
