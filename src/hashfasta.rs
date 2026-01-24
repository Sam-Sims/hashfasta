use crate::cli::{Command, CommonArgs, HashArgs, ModeArgs};
use crate::{hash, parser, Cli};
use color_eyre::eyre::eyre;
use color_eyre::Result;
use log::{info, warn};
use serde::Serialize;
use std::io::{self, Write};
use std::collections::HashMap;

#[derive(Serialize)]
struct RecordsPayload<'a, T> {
    records: &'a [T],
}

impl<'a> RecordsPayload<'a, parser::HashRecord> {
    fn from_records(records: &'a [parser::HashRecord]) -> Self {
        Self { records }
    }
}

#[derive(Serialize)]
struct HashPayload<'a> {
    #[serde(flatten)]
    records: RecordsPayload<'a, parser::HashRecord>,
    aggregate_hash: &'a str,
}

impl<'a> HashPayload<'a> {
    fn new(records: &'a [parser::HashRecord], aggregate_hash: &'a str) -> Self {
        Self {
            records: RecordsPayload::from_records(records),
            aggregate_hash,
        }
    }
}

#[derive(Serialize)]
struct DuplicateRecord<'a> {
    id: &'a str,
    hash: hash::SequenceHash,
    duplicate_of: &'a str,
    total_duplicates: usize,
}

fn warn_if_duplicates(records: &[parser::HashRecord]) {
    let hashes: Vec<hash::SequenceHash> = records.iter().map(|record| record.hash).collect();
    let counts = hash::count_duplicates(&hashes);
    if counts.values().any(|&count| count > 1) {
        warn!(
            "Detected duplicate sequences in input. Use the duplicates subcommand for more info."
        );
    }
}

pub struct Hashfasta {
    args: Cli,
}

impl Hashfasta {
    fn go_hash(
        args: &CommonArgs,
        force_single_thread: bool,
        include_ids: bool,
    ) -> Result<Vec<parser::HashRecord>> {
        let num_threads = if force_single_thread && args.threads > 1 {
            info!("Forcing --threads=1 to preserve input order for duplicate detection.");
            1
        } else {
            args.threads
        };

        parser::hash_fastx_file(
            &args.input,
            num_threads,
            args.normalise,
            args.canonical,
            include_ids,
            args.strict,
        )
    }

    fn record_id(record: &parser::HashRecord) -> Result<&str> {
        record
            .id
            .as_deref()
            // shouldnt really happen as we only call this when include_ids=true
            .ok_or_else(|| eyre!("Record is missing a FASTA/FASTQ header"))
    }

    fn run_duplicate(args: &ModeArgs) -> Result<()> {
        let records = Self::go_hash(&args.common, true, true)?;
        let hashes: Vec<hash::SequenceHash> = records.iter().map(|record| record.hash).collect();
        let counts = hash::count_duplicates(&hashes);
        let mut record_first_seen = HashMap::new();
        for record in &records {
            let id = Self::record_id(record)?;
            record_first_seen
                .entry(record.hash)
                .or_insert_with(|| id.to_string());
        }

        if args.common.json {
            let mut records_json = Vec::new();
            for record in &records {
                let count = counts[&record.hash];
                if count <= 1 {
                    continue;
                }
                let id = Self::record_id(record)?;
                let record_first_seen = record_first_seen[&record.hash].as_str();
                if record_first_seen == id {
                    continue;
                }
                records_json.push(DuplicateRecord {
                    id,
                    hash: record.hash,
                    duplicate_of: record_first_seen,
                    total_duplicates: count,
                });
            }
            let payload = RecordsPayload {
                records: &records_json,
            };
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            serde_json::to_writer_pretty(&mut handle, &payload)?;
            writeln!(handle)?;
            return Ok(());
        }

        println!("id\tduplicate_of_id\ttotal_duplicates\thash");
        for record in records {
            let count = counts[&record.hash];
            if count <= 1 {
                continue;
            }
            let id = Self::record_id(&record)?;
            let hash_hex = hash::hash_to_string(record.hash);
            let record_first_seen = record_first_seen[&record.hash].as_str();
            if record_first_seen == id {
                continue;
            }
            println!("{id}\t{record_first_seen}\t{count}\t{hash_hex}");
        }

        Ok(())
    }

    fn run_unique(args: &ModeArgs) -> Result<()> {
        let records = Self::go_hash(&args.common, true, true)?;
        let hashes: Vec<hash::SequenceHash> = records.iter().map(|record| record.hash).collect();
        let counts = hash::count_duplicates(&hashes);

        if args.common.json {
            let records_json: Vec<_> = records
                .iter()
                .filter(|record| counts[&record.hash] == 1)
                .collect();
            let payload = RecordsPayload {
                records: &records_json,
            };
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            serde_json::to_writer_pretty(&mut handle, &payload)?;
            writeln!(handle)?;
            return Ok(());
        }

        println!("id\thash");
        for record in records {
            if counts[&record.hash] == 1 {
                let id = Self::record_id(&record)?;
                let hash_hex = hash::hash_to_string(record.hash);
                println!("{id}\t{hash_hex}");
            }
        }

        Ok(())
    }

    fn run_hash(args: &HashArgs) -> Result<()> {
        let hash_records = Self::go_hash(&args.common, false, !args.quiet)?;
        let hashes: Vec<hash::SequenceHash> = hash_records.iter().map(|record| record.hash).collect();
        let final_hash = hash::calculate_final_hash(&hashes);

        // quiet mode only prints the final hash and exits
        if args.quiet {
            println!("{final_hash}");
            return Ok(());
        }

        if args.common.json {
            let records_slice: &[parser::HashRecord] = if args.quiet { &[] } else { &hash_records };
            let payload = HashPayload::new(records_slice, &final_hash);
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            serde_json::to_writer_pretty(&mut handle, &payload)?;
            writeln!(handle)?;
            return Ok(());
        }

        println!("id\thash");
        for record in &hash_records {
            let id = Self::record_id(record)?;
            let hash_string = hash::hash_to_string(record.hash);
            println!("{id}\t{hash_string}");
        }
        println!("aggregate_hash\t{final_hash}");
        warn_if_duplicates(&hash_records);

        Ok(())
    }

    pub fn run(&self) -> Result<()> {
        match &self.args.command {
            Command::Hash(args) => Self::run_hash(args),
            Command::Unique(args) => Self::run_unique(args),
            Command::Duplicate(args) => Self::run_duplicate(args),
        }
    }

    pub fn new(args: Cli) -> Self {
        Self { args }
    }
}

#[cfg(test)]
mod tests {
    use crate::hash;
    use crate::parser;
    use std::fs;
    use std::path::Path;
    use tempfile::{Builder, NamedTempFile};

    fn write_temp_file(contents: &str, suffix: &str) -> NamedTempFile {
        let file = Builder::new()
            .prefix("hashfasta_test_")
            .suffix(suffix)
            .tempfile()
            .expect("create temp test file");
        fs::write(file.path(), contents).expect("write temp test file");
        file
    }

    fn final_hash_for_path(path: &Path, normalise: bool, canonicalise: bool) -> String {
        let records = parser::hash_fastx_file(path, 1, normalise, canonicalise, false, false)
            .expect("process test file");
        let hashes: Vec<hash::SequenceHash> = records.iter().map(|record| record.hash).collect();
        hash::calculate_final_hash(&hashes)
    }

    #[test]
    fn test_hashes_match_valid() {
        let fasta_a = concat!(
            ">seq_one description\n",
            "ACGTACGT\n",
            ">seq_two other\n",
            "GGGTTTAA\n",
        );
        let fasta_b = concat!(
            ">another header\n",
            "GGGTTTAA\n",
            ">different name\n",
            "ACGTACGT\n",
        );
        let file_a = write_temp_file(fasta_a, ".fa");
        let file_b = write_temp_file(fasta_b, ".fa");

        let hash_a = final_hash_for_path(file_a.path(), false, false);
        let hash_b = final_hash_for_path(file_b.path(), false, false);

        assert_eq!(hash_a, hash_b);
    }

    #[test]
    fn test_hashes_match_fastq_valid() {
        let fastq_a = concat!(
            "@read_one\n",
            "ACGTACGT\n",
            "+\n",
            "!!!!!!!!\n",
            "@read_two\n",
            "GGGTTTAA\n",
            "+\n",
            "''''''''\n",
        );
        let fastq_b = concat!(
            "@different_header\n",
            "ACGTACGT\n",
            "+\n",
            "########\n",
            "@another_header\n",
            "GGGTTTAA\n",
            "+\n",
            "%%%%%%%%\n",
        );
        let file_a = write_temp_file(fastq_a, ".fq");
        let file_b = write_temp_file(fastq_b, ".fq");

        let hash_a = final_hash_for_path(file_a.path(), false, false);
        let hash_b = final_hash_for_path(file_b.path(), false, false);

        assert_eq!(hash_a, hash_b);
    }

    #[test]
    fn test_hashes_match_normalised_valid() {
        let fasta_a = concat!(">seq_one\n", "acgtuN-\n",);
        let fasta_b = concat!(">seq_two\n", "ACGTTn-\n",);
        let file_a = write_temp_file(fasta_a, ".fa");
        let file_b = write_temp_file(fasta_b, ".fa");

        let hash_a = final_hash_for_path(file_a.path(), true, false);
        let hash_b = final_hash_for_path(file_b.path(), true, false);

        assert_eq!(hash_a, hash_b);
    }

    #[test]
    fn test_hashes_match_canonicalised_valid() {
        let fasta_a = concat!(">seq_forward\n", "ACGTGGA\n",);
        let fasta_b = concat!(">seq_reverse_complement\n", "TCCACGT\n",);
        let file_a = write_temp_file(fasta_a, ".fa");
        let file_b = write_temp_file(fasta_b, ".fa");

        let hash_a = final_hash_for_path(file_a.path(), false, true);
        let hash_b = final_hash_for_path(file_b.path(), false, true);

        assert_eq!(hash_a, hash_b);
    }
}
