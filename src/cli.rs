use clap::Parser;

use crate::hashers::HashAlgorithm;

#[derive(Parser, Debug)]
#[command(
    author, version, about = "Quickly compute hashes for nucleotide sequences.", long_about = None
)]
pub struct Cli {
    /// Input FASTA or FASTQ file(s). Can be GZ. Use "-" for stdin.
    #[arg(
        value_name = "FASTA(s)", required_unless_present = "help", value_parser(check_input_exists)
    )]
    pub input: String,

    /// Output individual hashes for each sequence (TSV).
    #[arg(short = 'i', long = "individual", action, conflicts_with = "show_duplicates")]
    pub individual_output: bool,

    /// Considers the canonical sequence (the lexicographically smaller of the two reverse complementary sequences).
    #[arg(short = 'c', long = "canonical", action)]
    pub canonical: bool,

    /// Force the input to be treated as FASTA format.
    #[arg(long = "fasta", action)]
    pub fasta: bool,

    /// Force the input to be treated as FASTQ format.
    #[arg(long = "fastq", action)]
    pub fastq: bool,

    /// Output duplicates sequences (TSV).
    #[arg(short = 'd', long = "duplicates", action, conflicts_with = "individual_output")]
    pub show_duplicates: bool,

    /// Specify the algorithm to use for hashing sequences.
    #[arg(long = "seqhash", default_value = "highway")]
    pub seqhash: HashAlgorithm,

    /// Specify the algorithm to use for calculating the final hash.
    #[arg(long = "finalhash", default_value = "md5")]
    pub finalhash: HashAlgorithm,
}

fn check_input_exists(s: &str) -> Result<String, String> {
    if s == "-" {
        return Ok(s.to_string());
    }
    if std::path::Path::new(s).exists() {
        Ok(s.to_string())
    } else {
        Err(format!("File does not exist: {}", s))
    }
}
