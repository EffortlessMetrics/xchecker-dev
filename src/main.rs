//! xchecker CLI binary
//!
//! This is the minimal entrypoint for the xchecker CLI.
//! All logic is in the library; main.rs only invokes cli::run().

fn main() {
    // cli::run() handles ALL output including errors
    // Returns Result<(), ExitCode> - main only maps to process exit
    if let Err(code) = xchecker::cli::run() {
        std::process::exit(code.as_i32());
    }
}
