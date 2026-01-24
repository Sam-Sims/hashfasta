# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-01-24

### Added
- GitHub Actions workflows for `check`, `test`, and `release`
- Rewrite of entire core functionality:
- New subcommands: `hash`, `unique`, `duplicate`, replacing previous flag-based interface.
- Multi-threaded parsing
- Ability to read FASTX from stdin, http/https URLs, and ssh paths.
- Quiet mode (`--quiet`) to suppress per-record output in `hash` subcommand.
- JSON output mode (`--json`)
- Strict mode (`--strict`) to fail on non-ACGTUN- bases.


### Changed
- CLI interface is now subcommand-based
- Hashing now uses `xxhash3_64`
- Parsing switched from `noodles` to `paraseq` with parallel processing.

### Removed
- Selectable hash algorithms (`--seqhash`, `--finalhash`).
- CLI flags `--individual`, `--duplicates`, `--fasta`, and `--fastq` (replaced by subcommands).
