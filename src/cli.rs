use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    author, version, about = "Very quickly compute hashes for FASTX files considering **only** the sequence content.", long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(about = "Hash every record in the input and output a final aggregate_hash representing the sequence content of the entire file.")]
    Hash(HashArgs),
    #[command(about = "Output only the records whose sequences are unique within the input.")]
    Unique(ModeArgs),
    #[command(about = "Output only the records whose sequences are duplicates of earlier records.")]
    Duplicate(ModeArgs),
}

#[derive(Args, Debug, Clone)]
pub struct CommonArgs {
    #[arg(
        value_name = "FASTX",
        required_unless_present = "help",
        help = "Path/URL to input FASTA/FASTQ file, or - for stdin"
    )]
    pub input: String,

    #[arg(
        short = 'c',
        long = "canonical",
        help = "Use the canonical sequence (lexicographically smaller of forward and reverse complement)",
        action
    )]
    pub canonical: bool,

    #[arg(
        short = 'n',
        long = "normalise",
        alias = "normalize",
        help = "Normalise sequences before hashing",
        action
    )]
    pub normalise: bool,

    #[arg(
        short = 's',
        long = "strict",
        help = "Fail on non-ACGTUN- bases",
        action
    )]
    pub strict: bool,

    #[arg(
        short = 'j',
        long = "json",
        help = "Output records as JSON instead of TSV",
        action
    )]
    pub json: bool,

    #[arg(
        short = 't',
        long = "threads",
        default_value_t = 1,
        help = "Number of worker threads"
    )]
    pub threads: usize,
}

#[derive(Args, Debug, Clone)]
pub struct ModeArgs {
    #[command(flatten)]
    pub common: CommonArgs,
}

#[derive(Args, Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct HashArgs {
    #[command(flatten)]
    pub common: CommonArgs,

    #[arg(
        short = 'q',
        long = "quiet",
        help = "Only print the aggregate hash (suppress record-level output)",
        action
    )]
    pub quiet: bool,
}
