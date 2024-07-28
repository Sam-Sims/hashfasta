# hashfasta

Very quickly compute hashes from nucleotide sequences.

Supports `FASTA` and `FASTQ` files (and `gz` compressed versions). Supports reading from `stdin`.

## Overview

1. **Sequence processing**:
    - Converts all characters to uppercase
    - Masks any non-standard nucleotides (characters other than A, T, C, G, or N) as 'N'


2. **Individual Sequence Hashing**:
    - Computes a hash for each normalised sequence
    - fasta headers, sequence order, sequence names, and quality scores are ignored.
    - Provides options for considering canonical sequences
    - Allows selection of different hashing algorithms (HighwayHash, MD5, SHA2)


3. **File-Level Hashing**:
    - Generates a final hash by hashing the results from step 2

## Basic use cases:

- Generating a single hash for a dataset considering only on the nucleotide sequences

    ```
    hashfasta sequences.fasta
    ```
- Detecting duplicate sequences in a dataset

    ```
    hashfasta -d sequences.fasta > duplicates.tsv
    ```
- Hashing sequences from an archive, without decompressing to disk

    ```
    tar -xOf collection.tar.gz | hashfasta -
    ```

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