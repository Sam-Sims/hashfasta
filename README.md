![GitHub release (with filter)](https://img.shields.io/github/v/release/Sam-Sims/hashfasta)
![crates.io](https://img.shields.io/crates/v/hashfasta)
[![test](https://github.com/Sam-Sims/hashfasta/actions/workflows/test.yaml/badge.svg)](https://github.com/Sam-Sims/hashfasta/actions/workflows/test.yaml)
[![check](https://github.com/Sam-Sims/hashfasta/actions/workflows/check.yaml/badge.svg)](https://github.com/Sam-Sims/hashfasta/actions/workflows/check.yaml)
![MSRV](https://img.shields.io/badge/MSRV-1.87.0-blue)

# hashfasta

**hash** **fasta** (faster?)

Very quickly compute hashes for FASTA/FASTQ files considering **only** the sequence content.

Hashfasta produces a hash for an input file that is stable and dependent **only** on the sequence content. It ignores:
- FASTA/FASTQ headers
- Quality scores
- Read order

It can optionally consider only the canonical sequence (the lexicographically smaller of the forward and reverse complement) and/or normalise sequences before hashing. See [Sequence handling options](#sequence-handling-options).

It supports `FASTA` and `FASTQ` files (optionally compressed with `gz`) as input, and can read via http/https, SSH and stdin.

## Installation

### Binaries:

Precompiled binaries for Linux, MacOS and Windows are attached to the latest
release.

### Cargo:

Requires [cargo](https://www.rust-lang.org/tools/install)

```bash
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

All executables will be in the directory `hashfasta/target/release`.

## Usage
```bash
Very quickly compute hashes for FASTX files considering **only** the sequence content.

Usage: hashfasta <COMMAND>

Commands:
  hash       Hash every record in the input and output a final aggregate_hash representing the sequence content of the entire file.
  unique     Output only the records whose sequences are unique within the input.
  duplicate  Output only the records whose sequences are duplicates of earlier records.
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Subcommands

Each subcommand takes a single positional argument `<FASTX>` which is the path/URL to the input FASTA/FASTQ file (or `-` for stdin).
By default, output is written to stdout in tab-separated format. Use `--json` to output in JSON format. See [Output Formats](#output-format).

#### `hash`

Hash every record in the input and output a final "aggregate_hash" representing the seqeunce content of the entire file. Suppress per-record output with `-q/--quiet`.

```bash
hashfasta hash [OPTIONS] <FASTX>
```

#### `unique`

Output only the records whose sequences are unique within the input. Useful for deduplicating records.

```bash
hashfasta unique [OPTIONS] <FASTX>
```

#### `duplicate`

Output only the records whose sequences are duplicates of earlier records.

```bash
hashfasta duplicate [OPTIONS] <FASTX>
```

### Options

All subcommands share the following options:

- `-c, --canonical`:  Use the canonical sequence. See [Canonicalisation](#canonicalisation--c---canonical).
- `-n, --normalise`: Normalise sequences before hashing. See [Normalisation](#normalisation----normalise).
- `-s, --strict`: Fail if non-ACGTUN- bases are encountered.
- `-t, --threads`: Number of threads [default: 1].
- `-j, --json`: Output records as JSON instead of TSV.
- `-q, --quiet`: Only print the aggregate hash (suppress record-level output).

### Sequence handling options

#### Normalisation (`-n` / `--normalise`)

Normalisation ensures that sequences are treated consistently regardless of case or RNA/DNA differences. It performs the following:
- Converts all characters to uppercase.
- Converts Uracil (`U`) to Thymine (`T`).
- Masks any non-standard nucleotides (i.e characters other than `ACGTU-`) as `N`.

#### Canonicalisation (`-c` / `--canonical`)

Canonicalisation does the following:
1. Generates the reverse complement of the sequence.
2. Compares the original sequence with its reverse complement lexicographically.
3. Hashes the "smaller" of the two sequences.

This ensures that a sequence and its reverse complement will always produce the same hash.

### Examples

```bash
# Generate hashes for a single fasta file
hashfasta hash sequences.fasta

# Hash a compressed file normalising sequences, and consider canonical sequences
hashfasta hash -cn reads.fq.gz

# Fail early if invalid bases are encountered
hashfasta hash --strict input.fasta

# Output as JSON, instead of TSV
hashfasta hash --json input.fasta

# Read from stdin
tar -xOf collection.tar.gz | hashfasta hash -

# Read from HTTP
hashfasta hash https://example.com/sequences.fasta

# Read from SSH
hashfasta hash ssh://user@host/path/to/sequences.fasta
```

### Output Format

**Default (TSV):**
```tsv
id	hash
seq1	537edb87f29c16e5
seq2	2cbf6181d9bdb039
aggregate_hash	02402baf8722f975
```

**JSON (`--json`):**
```json
{
  "records": [
    {
      "id": "seq1",
      "hash": "537edb87f29c16e5"
    },
    {
      "id": "seq2",
      "hash": "2cbf6181d9bdb039"
    }
  ],
  "aggregate_hash": "02402baf8722f975"
}
```
