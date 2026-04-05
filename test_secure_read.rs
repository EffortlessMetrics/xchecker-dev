use std::fs;
use std::io::Read;
use std::path::Path;

pub fn secure_read_to_string<P: AsRef<Path>>(path: P) -> std::io::Result<String> {
    let mut file = fs::File::open(path)?;
    let metadata = file.metadata()?;

    if metadata.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Cannot read a directory",
        ));
    }

    let size = metadata.len();
    if size > 10 * 1024 * 1024 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "File too large (exceeds 10MB)",
        ));
    }

    let mut content = String::new();
    file.take(size + 1).read_to_string(&mut content)?;
    Ok(content)
}

fn main() {
    println!("hello");
}
