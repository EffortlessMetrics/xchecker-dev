use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

/// Maximum file size to read (10MB) to prevent DoS via huge files
pub const MAX_READ_SIZE: u64 = 10 * 1024 * 1024;

/// Securely read a file to a string, preventing TOCTOU and DoS via large files.
pub fn secure_read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let file = File::open(path)?;

    let metadata = file.metadata()?;
    if metadata.len() > MAX_READ_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File exceeds maximum allowed size",
        ));
    }

    let mut buffer = String::new();
    let bytes_read = file.take(MAX_READ_SIZE + 1).read_to_string(&mut buffer)?;

    if bytes_read > MAX_READ_SIZE as usize {
         return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File exceeds maximum allowed size",
        ));
    }

    Ok(buffer)
}
