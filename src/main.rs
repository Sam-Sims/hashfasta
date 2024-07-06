use std::fs::File;
use std::io::{self, BufRead, BufReader, Cursor, Read};

use anyhow::{Context, Result};
use clap::Parser;
use env_logger::Env;
use log::{error, info};
use owo_colors::{OwoColorize, Stream::Stdout};

use cli::Cli;
use hashers::calculate_final_hash;
use parser::{auto_determine_file_type, FileType};

mod cli;
mod hashers;
mod parser;


fn run_hash() -> Result<()> {
    let args = Cli::parse();
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "trace")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    for input_file in &args.input {
        let mut all_hashes = Vec::new();
        let mut reader: Box<dyn BufRead> = if input_file == "-" {
            let (reader, _) = niffler::get_reader(Box::new(io::stdin().lock())).unwrap();
            Box::new(BufReader::new(reader))
        } else {
            let (reader, _) = niffler::get_reader(Box::new(File::open(input_file)?)).unwrap();
            Box::new(BufReader::with_capacity(1024 * 64, reader))
        };
        info!("Processing file: {}", input_file);

        let file_type = if args.fasta {
            FileType::Fasta
        } else if args.fastq {
            FileType::Fastq
        } else {
            let mut first_lines = String::new();
            let mut line_count = 0;

            // Start iterating over the first 100 lines of the reader
            for line in reader.by_ref().lines() {
                let line = line?;
                first_lines.push_str(&line);
                first_lines.push('\n');
                line_count += 1;

                if line_count >= 100 {
                    break;
                }
            }
            // Send the first 100 lines to use in determining the file type
            let determined_type = auto_determine_file_type(&first_lines);
            // Create a new reader that combiness the first 100 lines we just read with the original reader
            let combined_reader = Cursor::new(first_lines).chain(reader);
            // Replace our original reader with this new combined reader
            reader = Box::new(BufReader::new(combined_reader));
            determined_type
        };

        let file_hashes = match file_type {
            FileType::Fasta => parser::fasta_reader(
                reader,
                args.individual_output,
                args.canonical,
                &args.seqhash,
            ).context("Error parsing FASTA file")?,
            FileType::Fastq => parser::fastq_reader(
                reader,
                args.individual_output,
                args.canonical,
                &args.seqhash,
            ).context("Error parsing FASTQ file")?,
            FileType::Unknown => {
                return Err(anyhow::anyhow!("Unable to determine the file type from the first 100 lines. Please specify --fasta or --fastq."));
            }
        };
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
fn main() {
    if let Err(err) = run_hash() {
        error!("{}", err);
        std::process::exit(1);
    }
}
