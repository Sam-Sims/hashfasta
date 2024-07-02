use std::fs::File;
use std::io::{self, BufRead, BufReader};

use clap::Parser;
use env_logger::Env;
use log::{info, warn};
use noodles::fasta;
use owo_colors::{OwoColorize, Stream::Stdout};

use cli::Cli;
use hashers::{calculate_final_hash, calculate_hash, HashAlgorithm};

mod cli;
mod hashers;

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

fn lookup(x: u8) -> u8 {
    LOOKUP_TABLE[x as usize]
}

fn rc_lookup(x: u8) -> u8 {
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

pub fn process_fasta_reader(
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
        let mut normal_seq = seq.iter().map(|&x| lookup(x)).collect::<Vec<u8>>();

        if reverse {
            let mut rc_seq = seq.iter().map(|&x| rc_lookup(x)).collect::<Vec<u8>>();
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

fn main() -> io::Result<()> {
    let args = Cli::parse();
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "trace")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    for input_file in &args.input {
        let mut all_hashes = Vec::new();
        let reader: Box<dyn BufRead> = if input_file == "-" {
            let (reader, _) = niffler::get_reader(Box::new(io::stdin().lock())).unwrap();
            Box::new(BufReader::new(reader))
        } else {
            let (reader, _) = niffler::get_reader(Box::new(File::open(input_file)?)).unwrap();
            Box::new(BufReader::new(reader))
        };

        // if args is fasta
        let filetype = match (args.fasta, args.fastq) {
            (true, _) => Filetype::Fasta,
            (_, true) => Filetype::Fastq,
            _ => Filetype::Auto,
        };

        info!("Processing file: {}", input_file);
        let file_hashes = process_fasta_reader(
            reader,
            args.individual_output,
            args.canonical,
            &args.seqhash,
        )?;
        all_hashes.extend(file_hashes);

        all_hashes.sort();
        let final_hash = calculate_final_hash(&args.finalhash, &all_hashes);

        println!(
            "Final hash\t{}",
            final_hash.if_supports_color(Stdout, |final_hash| final_hash.green())
        );
    }

    Ok(())
}
