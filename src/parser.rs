pub enum FileType {
    Fasta,
    Fastq,
    Unknown,
}

pub fn fasta_reader() {
    unimplemented!()
}

pub fn fastq_reader() {
    unimplemented!()
}

pub fn auto_determine_file_type(content: &str) -> FileType {
    let mut is_fastq = false;
    let mut is_fasta = false;

    for (i, line) in content.lines().enumerate() {
        if i % 4 == 0 && line.starts_with('@') {
            is_fastq = true;
            break;
        }
        if line.starts_with('>') {
            is_fasta = true;
        }
        if i >= 100 {
            break;
        }
    }

    if is_fastq {
        FileType::Fastq
    } else if is_fasta {
        FileType::Fasta
    } else {
        FileType::Unknown
    }
}