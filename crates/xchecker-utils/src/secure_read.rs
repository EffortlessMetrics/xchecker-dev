//! Secure file reading utilities to prevent TOCTOU and DoS vulnerabilities.

use std::fs;
use std::io::Read;
use std::path::Path;

/// Maximum file size to read (10MB) to prevent memory exhaustion / DoS
const MAX_READ_SIZE: u64 = 10 * 1024 * 1024;

/// Securely reads a file to a string, preventing TOCTOU and DoS vulnerabilities.
///
/// This function:
/// 1. Opens the file descriptor first
/// 2. Checks metadata on the opened file (preventing symlink replacement between check and open)
/// 3. Rejects directories gracefully
/// 4. Enforces a static 10MB limit to prevent memory exhaustion
/// 5. Uses `take()` as a bounded safety net
pub fn secure_read_to_string<P: AsRef<Path>>(path: P) -> std::io::Result<String> {
    let path = path.as_ref();

    // Open file first to prevent TOCTOU
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            // Check if error was because it's a directory (fallback)
            if let Ok(meta) = fs::metadata(path) {
                #[allow(clippy::collapsible_if)]
                if meta.is_dir() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Cannot read directory as file",
                    ));
                }
            }
            return Err(e);
        }
    };

    let metadata = file.metadata()?;

    if metadata.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Cannot read directory as file",
        ));
    }

    if metadata.len() > MAX_READ_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("File exceeds {} bytes limit", MAX_READ_SIZE),
        ));
    }

    let mut contents = String::new();
    // Use take() as a bounded safety net in case file grows while reading
    file.take(MAX_READ_SIZE).read_to_string(&mut contents)?;

    Ok(contents)
}
