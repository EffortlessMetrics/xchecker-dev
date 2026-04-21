sed -i 's/xchecker_utils::runner/xchecker_runner/g' crates/xchecker-runner/src/command_spec.rs
sed -i 's/xchecker_utils::runner/xchecker_runner/g' crates/xchecker-runner/src/native.rs
sed -i 's/xchecker_utils::runner/xchecker_runner/g' crates/xchecker-runner/src/process.rs
sed -i 's/xchecker_utils::error::RunnerError/xchecker_runner::RunnerError/g' crates/xchecker-runner/src/process.rs
sed -i 's/xchecker_utils::runner/xchecker_runner/g' crates/xchecker-runner/src/wsl.rs
