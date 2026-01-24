use crate::hash::{self, SequenceHash};
use color_eyre::Result;
use paraseq::prelude::*;
use paraseq::Result as ParaseqResult;
use paraseq::{fastx, BoxedReader};
use parking_lot::Mutex;
use serde::Serialize;
use std::path::Path;
use std::sync::Arc;

#[derive(Clone, Serialize)]
pub struct HashRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub hash: SequenceHash,
}

#[derive(Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct RecordProcessor {
    pub results: Arc<Mutex<Vec<HashRecord>>>,
    pub normalise: bool,
    pub canonicalise: bool,
    pub include_ids: bool,
    pub strict: bool,
}

impl RecordProcessor {
    #[allow(clippy::fn_params_excessive_bools)]
    pub fn new(normalise: bool, canonical: bool, include_ids: bool, strict: bool) -> Self {
        Self {
            results: Arc::new(Mutex::new(Vec::new())),
            normalise,
            canonicalise: canonical,
            include_ids,
            strict,
        }
    }
}

impl<Rf: Record> ParallelProcessor<Rf> for RecordProcessor {
    fn process_record(&mut self, record: Rf) -> ParaseqResult<()> {
        let seq = record.seq();
        if self.strict {
            validate_sequence(seq.as_ref(), record.id())?;
        }
        let hash = hash::hash_sequence_bytes(seq.as_ref(), self.normalise, self.canonicalise);
        let id = if self.include_ids {
            Some(String::from_utf8_lossy(record.id()).to_string())
        } else {
            None
        };

        let mut lock = self.results.lock();
        lock.push(HashRecord { id, hash });

        Ok(())
    }
}

fn is_valid_base(base: u8) -> bool {
    matches!(
        base,
        b'A' | b'C' | b'G' | b'T' | b'U' | b'N' | b'-' | b'a' | b'c' | b'g' | b't' | b'u' | b'n'
    )
}

fn validate_sequence(seq: &[u8], id: &[u8]) -> ParaseqResult<()> {
    for &base in seq {
        if !is_valid_base(base) {
            let id_display = String::from_utf8_lossy(id).to_string();
            let message = format!("Record {id_display} contains an invalid base.");
            let err = std::io::Error::new(std::io::ErrorKind::InvalidData, message);
            return Err(err.into());
        }
    }
    Ok(())
}

fn is_http_url(input: &str) -> bool {
    input.starts_with("http://") || input.starts_with("https://")
}

fn is_ssh_path(input: &str) -> bool {
    input.starts_with("ssh://")
}

fn open_fastx_reader(input: &str) -> Result<fastx::Reader<BoxedReader>> {
    if input == "-" {
        return Ok(fastx::Reader::from_stdin()?);
    }

    if is_http_url(input) {
        return Ok(fastx::Reader::from_url(input)?);
    }

    if is_ssh_path(input) {
        return Ok(fastx::Reader::from_ssh(input)?);
    }

    Ok(fastx::Reader::from_path(Path::new(input))?)
}

#[allow(clippy::fn_params_excessive_bools)]
pub fn hash_fastx_file<P: AsRef<Path>>(
    path: P,
    num_threads: usize,
    normalise: bool,
    canonical: bool,
    include_ids: bool,
    strict: bool,
) -> Result<Vec<HashRecord>> {
    let path_ref = path.as_ref();
    let reader = if let Some(path_str) = path_ref.to_str() {
        open_fastx_reader(path_str)?
    } else if path_ref == Path::new("-") {
        fastx::Reader::from_stdin()?
    } else {
        fastx::Reader::from_path(path_ref)?
    };

    let mut processor = RecordProcessor::new(normalise, canonical, include_ids, strict);

    reader.process_parallel(&mut processor, num_threads)?;

    let final_hashes = processor.results.lock().clone();
    Ok(final_hashes)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_is_valid_base_valid() {
        let valid_bases = b"ACGTUN-acgtnu";
        for &base in valid_bases {
            assert!(
                super::is_valid_base(base),
                "Base {} should be valid",
                base as char
            );
        }
    }

    #[test]
    fn test_is_valid_base_invalid() {
        let invalid_bases = b"XYZ#@!123";
        for &base in invalid_bases {
            assert!(
                !super::is_valid_base(base),
                "Base {} should be invalid",
                base as char
            );
        }
    }

    #[test]
    fn test_is_ssh_path_only_accepts_ssh_scheme() {
        assert!(super::is_ssh_path(
            "ssh://user@example.com/path/to/file.fastq"
        ));
        assert!(super::is_ssh_path(
            "ssh://example.com:2222/path/to/file.fastq"
        ));
        assert!(!super::is_ssh_path("/tmp/file.fastq"));
        assert!(!super::is_ssh_path("./relative.fastq"));
        assert!(!super::is_ssh_path("../relative.fastq"));
    }

    #[test]
    fn test_is_http_url_only_accepts_http_and_https() {
        assert!(super::is_http_url("http://example.com/file.fastq"));
        assert!(super::is_http_url("https://example.com/file.fastq"));
        assert!(!super::is_http_url("/blah/blah.fastq"));
        assert!(!super::is_http_url("./blah.fastq"));
        assert!(!super::is_http_url("../blah.fastq"));
    }
}
