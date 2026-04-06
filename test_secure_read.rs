use std::fs;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB

pub fn secure_read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let path = path.as_ref();

    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            if let Ok(meta) = fs::metadata(path) {
                if meta.is_dir() {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, "Is a directory"));
                }
            }
            return Err(e);
        }
    };

    let metadata = file.metadata()?;
    if metadata.is_dir() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Is a directory"));
    }

    if metadata.len() > MAX_FILE_SIZE {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "File exceeds 10MB limit"));
    }

    let mut content = String::new();
    file.take(MAX_FILE_SIZE).read_to_string(&mut content)?;
    Ok(content)
}

fn main() {
    match secure_read_to_string("Cargo.toml") {
        Ok(content) => println!("Read Cargo.toml: {} bytes", content.len()),
        Err(e) => println!("Error reading Cargo.toml: {}", e),
    }
    match secure_read_to_string("crates") {
        Ok(content) => println!("Read crates: {} bytes", content.len()),
        Err(e) => println!("Error reading crates: {}", e),
    }
}
