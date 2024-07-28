use std::io::BufRead;

use anyhow::{Context, Result};
use noodles::fasta;
use noodles::fastq;

use crate::hashers::{calculate_hash, HashAlgorithm};

const LOOKUP_TABLE: [u8; 256] = [
    //A = 1 C = 2 G = 3 T = 4
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const RC_LOOKUP_TABLE: [u8; 256] = [
    //A = 1 C = 2 G = 3 T = 4
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 4, 0, 3, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 4, 0, 3, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

#[inline(always)]
pub fn lookup(x: u8) -> u8 {
    LOOKUP_TABLE[x as usize]
}

#[inline(always)]
pub fn rc_lookup(x: u8) -> u8 {
    RC_LOOKUP_TABLE[x as usize]
}

/// Trait to allow trimming ascii whitespace from a &[u8].
pub trait SliceExt {
    fn trim(&self) -> &Self;
}

impl SliceExt for [u8] {
    /// https://stackoverflow.com/questions/31101915/how-to-implement-trim-for-vecu8
    ///
    /// Trim ascii whitespace (based on is_ascii_whitespace())
    /// from the start and end of &\[u8\].
    ///
    /// Returns &\[u8\] with leading and trailing whitespace removed.
    fn trim(&self) -> &[u8] {
        let from = match self.iter().position(|x| !x.is_ascii_whitespace()) {
            Some(i) => i,
            None => return &self[0..0],
        };
        let to = self.iter().rposition(|x| !x.is_ascii_whitespace()).unwrap();
        &self[from..=to]
    }
}

pub enum FileType {
    Fasta,
    Fastq,
    Unknown,
}

pub fn fasta_reader(
    reader: Box<dyn BufRead>,
    canonical: bool,
    algorithm: &HashAlgorithm,
) -> Result<Vec<(String, String)>> {
    let mut reader = fasta::Reader::new(reader);
    let mut hashes = Vec::new();
    let mut buffer = Vec::new();

    for result in reader.records() {
        let record = result.context("Error reading FASTA record")?;
        let record_name = String::from_utf8_lossy(record.name())
            .to_string();
        let seq = record.sequence().as_ref().trim();

        buffer.clear();
        buffer.reserve(seq.len());
        buffer.extend(seq.iter().map(|&x| lookup(x)));

        if canonical {
            let mut rc_seq: Vec<_> = seq.iter().map(|&x| rc_lookup(x)).collect();
            rc_seq.reverse();
            if rc_seq < buffer {
                buffer = rc_seq;
            }
        }

        let hash = calculate_hash(algorithm, &buffer);
        hashes.push((record_name, hash));
    }

    Ok(hashes)
}


pub fn fastq_reader(
    reader: Box<dyn BufRead>,
    canonical: bool,
    algorithm: &HashAlgorithm,
) -> Result<Vec<(String, String)>> {
    let mut reader = fastq::Reader::new(reader);
    let mut hashes = Vec::new();
    let mut buffer = Vec::new();

    for result in reader.records() {
        let record = result.context("Error reading FASTQ record")?;
        let record_name = String::from_utf8_lossy(record.name()).into_owned();
        let seq = record.sequence().trim();

        buffer.clear();
        buffer.reserve(seq.len());
        buffer.extend(seq.iter().map(|&x| lookup(x)));

        if canonical {
            let mut rc_seq: Vec<_> = seq.iter().map(|&x| rc_lookup(x)).collect();
            rc_seq.reverse();
            if rc_seq < buffer {
                buffer = rc_seq;
            }
        }

        let hash = calculate_hash(algorithm, &buffer);
        hashes.push((record_name, hash));
    }
    Ok(hashes)
}

pub fn auto_determine_file_type(content: &str) -> FileType {
    let mut is_fastq = false;
    let mut is_fasta = false;

    for (i, line) in content.lines().enumerate() {
        if i % 4 == 0 && line.starts_with('@') {
            is_fastq = true;
            break;
        }
        if line.starts_with('>') {
            is_fasta = true;
        }
        if i >= 100 {
            break;
        }
    }

    if is_fastq {
        FileType::Fastq
    } else if is_fasta {
        FileType::Fasta
    } else {
        FileType::Unknown
    }
}
