pub(crate) fn should_run_e2e() -> bool {
    if std::env::var_os("XCHECKER_E2E").is_none() {
        return false;
    }

    claude_stub_path().is_some()
}

pub(crate) fn claude_stub_path() -> Option<String> {
    // Cargo converts hyphens to underscores in CARGO_BIN_EXE_* env vars.
    // So `claude-stub` binary becomes `CARGO_BIN_EXE_claude_stub`.
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_claude_stub") {
        return Some(path);
    }

    // Back-compat: check hyphen form in case older scripts set it directly
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_claude-stub") {
        return Some(path);
    }

    // PATH fallbacks: check for claude-stub binary, then fall back to claude
    which::which("claude-stub")
        .or_else(|_| which::which("claude"))
        .ok()
        .map(|path| path.to_string_lossy().to_string())
}
