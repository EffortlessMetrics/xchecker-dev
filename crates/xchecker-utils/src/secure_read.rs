use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB

pub fn secure_read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;

    if metadata.is_dir() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Path is a directory"));
    }

    if metadata.len() > MAX_FILE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File exceeds maximum allowed size of 10MB",
        ));
    }

    let mut content = String::new();
    file.take(MAX_FILE_SIZE).read_to_string(&mut content)?;

    Ok(content)
}
