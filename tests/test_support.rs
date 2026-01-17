use std::path::PathBuf;

pub(crate) fn should_run_e2e() -> bool {
    if std::env::var_os("XCHECKER_E2E").is_none() {
        return false;
    }

    if std::env::var_os("CARGO_BIN_EXE_claude-stub").is_some()
        || which::which("claude-stub").is_ok()
    {
        return true;
    }

    which::which("claude").is_ok()
}

pub(crate) fn claude_stub_path() -> String {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_claude-stub") {
        return path;
    }

    if let Ok(path) = which::which("claude-stub") {
        return path.to_string_lossy().to_string();
    }

    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    format!(
        "cargo run --manifest-path \"{}\" --bin claude-stub --",
        manifest_path.display()
    )
}
