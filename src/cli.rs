use clap::Parser;

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

    //run md5
    #[arg(long = "md5", action)]
    pub md5: bool,

    //run sha1
    #[arg(long = "sha2", action)]
    pub sha2: bool,

    //run highwayhash
    #[arg(long = "highway", action)]
    pub highway: bool,
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
