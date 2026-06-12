The failures are:
1) `test_sigterm_then_sigkill_sequence`: Panics at "Process should be terminated after SIGKILL"
2) `test_graceful_termination_with_sigterm`: Panics at "Process should be terminated after SIGTERM"
3) `test_process_group_termination`: Panics at "Parent process should be terminated"
4) `test_timeout_grace_period`: Panics at "Process should be terminated after SIGKILL"
5) `test_runner_timeout_terminates_process_group`: Panics at `Expected timeout error, got: NativeExecutionFailed { reason: "Failed to spawn claude process: No such file or directory (os error 2)" }`

For 1, 2, 3, 4:
The issue is that `sleep 30` or `sh -c` are executing. In Unix, when we execute `sleep(Duration::from_millis(500)).await;` to wait for termination after sending a signal, it might not be enough time, OR the process wasn't actually fully set up before the signal was sent.
Wait, let's look at the memory:
`test_runner_timeout_terminates_process_group`:
"When testing timeout behavior in `tests/test_unix_process_termination.rs` (e.g., `test_runner_timeout_terminates_process_group`), ensure the mocked executable path is set to a valid system binary (e.g., `runner.wsl_options.claude_path = Some("bash".to_string())`) to prevent 'No such file or directory' (os error 2) panics during process spawning." Wait, `claude_path` in `wsl_options` but it's a native runner? Let's check `Runner::native()`. No, `runner.claude_path` needs to be `bash`? Wait, `execute_claude` executes the `claude_path`. By default, `claude_path` might be `"claude"`. If we just set `runner.claude_path = Some("bash".to_string());` it should fix 5.

For 1, 2, 3, 4:
Memory:
"Tests spawning shell commands to ignore or handle signals (e.g., `test_sigterm_then_sigkill_sequence` and `test_graceful_termination_with_sigterm` in `tests/test_unix_process_termination.rs`) must use `sh` with a `while` loop (e.g., `trap 'exit 0' TERM; while true; do sleep 1; done`) rather than a bare `sleep 30` to prevent shell optimization via `exec` bypassing the trap/handler and explicitly handle the signal. They also require a Rust-side delay (e.g., 1000ms) immediately after `cmd.spawn()` to allow signal handler registration before sending signals, and an increased `sleep(Duration::from_millis(2000)).await` after `killpg` to prevent race conditions during CI verification checks."

Ah!
So I need to:
1. Fix `test_sigterm_then_sigkill_sequence`:
Change `.arg("trap '' TERM; sleep 30")` to `.arg("trap '' TERM; while true; do sleep 1; done")`
Add `sleep(Duration::from_millis(1000)).await;` right after `cmd.spawn()?`.
Increase sleep after `killpg` to `2000` (e.g., `sleep(Duration::from_millis(2000)).await;`).

2. Fix `test_graceful_termination_with_sigterm`:
Change `CommandSpec::new("sleep").arg("30")` to `CommandSpec::new("sh").arg("-c").arg("trap 'exit 0' TERM; while true; do sleep 1; done")`
Add `1000ms` sleep after `cmd.spawn()?`.
Change `sleep(Duration::from_millis(500)).await;` after `killpg` to `2000ms`.

3. Fix `test_process_group_termination`:
Add `sleep(Duration::from_millis(1000)).await;` after `cmd.spawn()?` (already has 500ms, change to 1000).
Change `sleep(Duration::from_millis(500)).await;` after `killpg` to `2000ms`.

4. Fix `test_timeout_grace_period`:
Change `CommandSpec::new("sleep").arg("30")` to `CommandSpec::new("sh").arg("-c").arg("trap 'exit 0' TERM; while true; do sleep 1; done")`
Add `1000ms` sleep after `cmd.spawn()?`.
After `killpg`, wait `2000ms` (the test already waits 5s, then sends SIGKILL, then waits 500ms. I'll change the 500ms to 2000ms).

5. Fix `test_runner_timeout_terminates_process_group`:
Set `runner.claude_path = "bash".into();`? No, let's read the struct `Runner`.
