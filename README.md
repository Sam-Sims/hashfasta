# hashfasta

Very quickly compute hashes from nucleotide sequences. Supports both `fasta` and `fastq` files (and their `.gz`
alternatives). Also supports reading from `stdin`

In brief:

1. Normalise the sequence (convert to uppercase, mask non `ATCGN` characters as `N`)
2. Compute the hash for each sequence
3. Compute the hash for the entire file by hashing the results of step 2

Fasta headers, sequence order, sequence names, and quality scores are ignored. Only the nucleotide sequence is
considered in the hash. By default, sequences are hashed using [HighwayHash](https://github.com/nickbabcock/highway-rs)
and a final `MD5` hash is produced from each hashed sequence. The hashing algorithms for each step can be changed
using `--seqhash` and `--finalhash`
respectively. Supported hashing algorithms are `SHA2`, `MD5` and `HighwayHash`.

## Installation

### Binaries:

Precompiled binaries for Linux, MacOS and Windows are attached to the latest
release [0.1.0](https://github.com/Sam-Sims/ambigviz/releases/tag/v0.1.0)

### Cargo:

Requires [cargo](https://www.rust-lang.org/tools/install)

```
cargo install hashfasta
```

### Build from source:

#### Install rust toolchain:

To install please refer to the rust documentation: [docs](https://www.rust-lang.org/tools/install)

#### Clone the repository:

```bash
git clone https://github.com/Sam-Sims/hashfasta
```

#### Build and add to path:

```bash
cd hashfasta
cargo build --release
export PATH=$PATH:$(pwd)/target/release
```

All executables will be in the directory hashfasta/target/release.

## Usage

### Basic usage:

```
Quickly compute hashes for nucleotide sequences.

Usage: hashfasta [OPTIONS] <FASTA(s)>

Arguments:
  <FASTA(s)>  Input FASTA or FASTQ file(s). Can be GZ. Use "-" for stdin

Options:
  -i, --individual             Output individual hashes for each sequence (TSV)
  -c, --canonical              Considers the canonical sequence (the lexicographically smaller of the two reverse complementary sequences) when hashing
      --fasta                  Force the input to be treated as FASTA format
      --fastq                  Force the input to be treated as FASTQ format
  -d, --duplicates             Output duplicates sequences (TSV)
      --seqhash <SEQHASH>      Specify the algorithm to use for hashing sequences [default: highway] [possible values: highway, md5, sha2]
      --finalhash <FINALHASH>  Specify the algorithm to use for calculating the final hash [default: md5] [possible values: highway, md5, sha2]
  -h, --help                   Print help
  -V, --version                Print version

```