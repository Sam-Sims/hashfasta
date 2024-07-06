use clap::Parser;

use crate::hashers::HashAlgorithm;

#[derive(Parser, Debug)]
#[command(version, about = "Hash")]
pub struct Cli {
    // read input, store multiple files in a vector
    #[arg(value_name = "FASTA(s)", value_parser(check_input_exists))]
    pub input: Vec<String>,

    // output individual hashes
    #[arg(short = 'i', long = "individual", action)]
    pub individual_output: bool,

    //reverse complement mode
    #[arg(short = 'c', long = "canonical", action)]
    pub canonical: bool,

    // fasta
    #[arg(long = "fasta", action)]
    pub fasta: bool,

    // fastq
    #[arg(long = "fastq", action)]
    pub fastq: bool,

    //display duplicates
    #[arg(short = 'd', long = "duplicates", action)]
    pub show_duplicates: bool,

    // sequence hash algorithm
    #[arg(long = "seqhash", default_value = "highway")]
    pub seqhash: HashAlgorithm,

    // final hash algorithm
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
