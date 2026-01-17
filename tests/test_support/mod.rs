pub(crate) fn should_run_e2e() -> bool {
    if std::env::var_os("XCHECKER_E2E").is_none() {
        return false;
    }

    std::env::var_os("CARGO_BIN_EXE_claude-stub").is_some()
        || which::which("claude-stub").is_ok()
        || which::which("claude").is_ok()
}

pub(crate) fn claude_stub_path() -> Option<String> {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_claude-stub") {
        return Some(path);
    }

    which::which("claude-stub")
        .ok()
        .map(|path| path.to_string_lossy().to_string())
}
