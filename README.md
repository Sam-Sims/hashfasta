# hashfasta

Very quickly compute hashes from nucleotide sequences. Supports both `fasta` and `fastq` files (and their `.gz`
alternatives) or `stdin`.

1. Normalise the sequence (convert to uppercase, mask non `ATCGN` characters as `N`)
2. Compute the hash for each sequence
3. Compute the hash for the entire file by hashing the results of step 2

Fasta headers, sequence order, sequence names, and quality scores are ignored. Only the nucleotide sequence is
considered in the hash. By default outputs the final hash, but can output individual hashes with `-i`. Outputs are in
tab-separated format with the sequence name in the first column and the hash in the second.

## Hashing algorithms

The code provides functionality for hashing sequences using different algorithms. By default, SHA-2 is used for hashing,
but users can specify MD5 or [HighwayHash](https://github.com/nickbabcock/highway-rs) through command-line options.

Available Hashing Algorithms

    SHA-2 (default):
        Produces a 256-bit hash value.
        Used when no specific algorithm is specified by the user.

    MD5:
        Produces a 128-bit hash value.
        Can be used by specifying the --md5 option in the command line.

    HighwayHash:
        Produces a 64-bit hash value.
        Can be used by specifying the --highway option in the command line.

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

```bash
hashfasta <fast[a|q]> [options]
```

### Examples:

You compress several fasta files into a tar archive and want to ensure that nothing was changed:

```bash
tar -xOvf collection.tar.gz | hash -
```

You decompress several fasta files from a tar archive and want to ensure that nothing was changed:

```bash
cat *.fasta | hash -
```

You want to compute an individual checksum for several fasta files

```bash
hash *.fasta
```

### Options:

### Input

The input can

### Canonical

`-c, --canonical` | Default: `false`

The canonical option considers the canonical sequence (the lexicographically smaller of the two reverse complementary
sequences) when hashing.