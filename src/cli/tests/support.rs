//! Test support utilities for CLI tests
//!
//! This module provides common test infrastructure including environment
//! isolation and mutex guards for tests that mutate global state.

use std::env;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};
use tempfile::TempDir;

// Global lock for tests that mutate process-global CLI state (env vars, cwd).
// Any test that uses `TestEnvGuard` or `cli_env_guard()` will be serialized.
static CLI_ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub fn cli_env_guard() -> MutexGuard<'static, ()> {
    CLI_ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

pub struct TestEnvGuard {
    // Hold the lock for the entire lifetime of the guard
    _lock: MutexGuard<'static, ()>,
    _temp_dir: TempDir,
    original_dir: PathBuf,
    original_xchecker_home: Option<String>,
}

impl Drop for TestEnvGuard {
    fn drop(&mut self) {
        // Restore env and cwd while still holding the lock
        match &self.original_xchecker_home {
            Some(val) => unsafe { env::set_var("XCHECKER_HOME", val) },
            None => unsafe { env::remove_var("XCHECKER_HOME") },
        }
        let _ = env::set_current_dir(&self.original_dir);
        // _lock field drops last, releasing the mutex
    }
}

pub fn setup_test_environment() -> TestEnvGuard {
    // Take the global CLI lock first
    let lock = cli_env_guard();

    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();
    let original_xchecker_home = env::var("XCHECKER_HOME").ok();

    // From here onwards we're serialized against other CLI tests
    env::set_current_dir(temp_dir.path()).unwrap();

    // CRITICAL: Set XCHECKER_HOME to the temp directory to ensure complete isolation.
    // Without this, tests may read/write against a developer machine's environment.
    // Safety: We hold the global CLI lock, so no concurrent test can race on env vars.
    unsafe {
        env::set_var("XCHECKER_HOME", temp_dir.path());
    }

    TestEnvGuard {
        _lock: lock,
        _temp_dir: temp_dir,
        original_dir,
        original_xchecker_home,
    }
}
