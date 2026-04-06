use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

/// Maximum allowed file size to read (10 MB)
const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// Securely reads a file into a string.
///
/// Mitigates TOCTOU vulnerabilities and prevents DoS from unbounded file sizes.
/// - Opens the file descriptor first before checking metadata.
/// - Returns an error if the path is a directory.
/// - Imposes a strict 10MB limit.
pub fn secure_read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let path = path.as_ref();

    // Open the file descriptor first to prevent TOCTOU
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            // Check if error was because it's a directory
            #[allow(clippy::collapsible_if)]
            if let Ok(meta) = fs::metadata(path) {
                if meta.is_dir() {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, "Is a directory"));
                }
            }
            return Err(e);
        }
    };

    // Check metadata securely using the file descriptor
    let metadata = file.metadata()?;

    if metadata.is_dir() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Is a directory"));
    }

    if metadata.len() > MAX_FILE_SIZE {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "File exceeds 10MB limit"));
    }

    // Use take() on the file descriptor to ensure we bound the read, consuming the file by value
    // to avoid clippy::unused_mut.
    let mut content = String::new();
    file.take(MAX_FILE_SIZE).read_to_string(&mut content)?;

    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_secure_read_success() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.txt");
        fs::write(&path, "hello world").unwrap();

        let content = secure_read_to_string(&path).unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_secure_read_directory_fails() {
        let temp = TempDir::new().unwrap();
        let err = secure_read_to_string(temp.path()).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn test_secure_read_not_found() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("does_not_exist.txt");
        let err = secure_read_to_string(&path).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }
}
