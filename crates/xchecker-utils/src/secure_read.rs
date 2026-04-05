use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB

pub fn secure_read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let path = path.as_ref();
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            if std::fs::metadata(path).is_ok_and(|meta| meta.is_dir()) {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "Cannot read a directory"));
            }
            return Err(e);
        }
    };

    let metadata = file.metadata()?;
    if metadata.is_dir() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Cannot read a directory"));
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
