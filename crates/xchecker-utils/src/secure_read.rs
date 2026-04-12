use std::fs::File;
use std::io::{Error, ErrorKind, Read};
use std::path::Path;

/// Reads the entire contents of a file into a string securely.
///
/// This function protects against TOCTOU (Time-of-Check to Time-of-Use) vulnerabilities
/// by opening the file descriptor first before inspecting metadata. It also prevents DoS
/// via memory exhaustion by limiting the read to 10MB.
pub fn secure_read_to_string<P: AsRef<Path>>(path: P) -> std::io::Result<String> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;

    if metadata.is_dir() {
        return Err(Error::new(ErrorKind::InvalidData, "Cannot read a directory"));
    }

    const MAX_SIZE: u64 = 10 * 1024 * 1024; // 10MB limit

    if metadata.len() > MAX_SIZE {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "File too large (exceeds 10MB)",
        ));
    }

    let mut content = String::new();
    file.take(MAX_SIZE).read_to_string(&mut content)?;

    Ok(content)
}
