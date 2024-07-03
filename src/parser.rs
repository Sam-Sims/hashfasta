use std::collections::HashSet;
use std::io;
use std::io::BufRead;

use log::warn;
use noodles::fasta;
use noodles::fastq;
use owo_colors::OwoColorize;
use owo_colors::Stream::Stdout;

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

pub fn lookup(x: u8) -> u8 {
    LOOKUP_TABLE[x as usize]
}

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
    output_individual: bool,
    reverse: bool,
    algorithm: &HashAlgorithm,
) -> io::Result<Vec<String>> {
    let mut reader = fasta::Reader::new(reader);
    let mut hashes = Vec::new();
    let mut duplicates = Vec::new();

    for result in reader.records() {
        let record = result?;
        let record_name = std::str::from_utf8(record.name()).unwrap().to_string();
        let seq = record.sequence().as_ref().trim();
        let mut normal_seq = Vec::with_capacity(seq.len());
        for &x in seq {
            normal_seq.push(lookup(x));
        }

        if reverse {
            let mut rc_seq = Vec::with_capacity(seq.len());
            for &x in seq {
                rc_seq.push(rc_lookup(x));
            }
            rc_seq.reverse();
            if rc_seq < normal_seq {
                normal_seq = rc_seq;
            }
        }
        let hash = calculate_hash(algorithm, &normal_seq);
        if output_individual {
            println!(
                "{}\t{}",
                record_name.if_supports_color(Stdout, |record_name| record_name.white()),
                hash.if_supports_color(Stdout, |hash| hash.green())
            );
        }
        if hashes.contains(&hash) {
            duplicates.push(record_name);
        }
        hashes.push(hash);
    }
    if !duplicates.is_empty() {
        warn!("Duplicates found:");
        for duplicate in duplicates {
            warn!("{}", duplicate);
        }
    }
    Ok(hashes)
}

pub fn fastq_reader(
    reader: Box<dyn BufRead>,
    output_individual: bool,
    reverse: bool,
    algorithm: &HashAlgorithm,
) -> io::Result<Vec<String>> {
    let mut reader = fastq::Reader::new(reader);
    let mut hashes = HashSet::new();
    let mut duplicates = Vec::new();

    for result in reader.records() {
        let record = result?;
        let record_name = std::str::from_utf8(record.name()).unwrap().to_string();
        let seq = record.sequence().trim();
        let mut normal_seq = Vec::with_capacity(seq.len());
        for &x in seq {
            normal_seq.push(lookup(x));
        }

        if reverse {
            let mut rc_seq = Vec::with_capacity(seq.len());
            for &x in seq {
                rc_seq.push(rc_lookup(x));
            }
            rc_seq.reverse();
            if rc_seq < normal_seq {
                normal_seq = rc_seq;
            }
        }

        // we add this to a hashset, so seems inefficient to calculate the hash twice
        let hash = calculate_hash(algorithm, &normal_seq);

        if output_individual {
            println!(
                "{}\t{}",
                record_name.if_supports_color(Stdout, |record_name| record_name.white()),
                hash.if_supports_color(Stdout, |hash| hash.green())
            );
        }

        if !hashes.insert(hash.clone()) {
            duplicates.push(record_name);
        }
    }

    if !duplicates.is_empty() {
        warn!("Duplicates found:");
        for duplicate in duplicates {
            warn!("{}", duplicate);
        }
    }
    Ok(hashes.into_iter().collect())
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
