use std::fs;
use std::io::{self, Read};
use std::path::Path;

/// Maximum reasonable size for files we process (10MB) to prevent DoS
pub const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// Securely reads a file into a string, preventing TOCTOU and DoS attacks.
pub fn secure_read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let file = fs::File::open(&path)?;
    let metadata = file.metadata()?;

    if metadata.is_dir() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Is a directory"));
    }

    if metadata.len() > MAX_FILE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("File exceeds maximum size of {} bytes", MAX_FILE_SIZE),
        ));
    }

    let mut content = String::new();
    file.take(MAX_FILE_SIZE).read_to_string(&mut content)?;
    Ok(content)
}
