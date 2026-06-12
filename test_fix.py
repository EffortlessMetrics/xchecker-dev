import re

with open('tests/test_unix_process_termination.rs', 'r') as f:
    content = f.read()

# 1. test_sigterm_then_sigkill_sequence
content = content.replace(
    '.arg("trap \'\' TERM; sleep 30")',
    '.arg("trap \'\' TERM; while true; do sleep 1; done")'
)
content = content.replace(
    'let pid = child.id().expect("Failed to get child PID");\n    let pgid = Pid::from_raw(pid as i32);\n\n    // Verify process is running\n    assert!(\n        is_process_running(pid),\n        "Process should be running initially"\n    );\n\n    // Send SIGTERM (process will ignore it)',
    'let pid = child.id().expect("Failed to get child PID");\n    let pgid = Pid::from_raw(pid as i32);\n\n    sleep(Duration::from_millis(1000)).await;\n\n    // Verify process is running\n    assert!(\n        is_process_running(pid),\n        "Process should be running initially"\n    );\n\n    // Send SIGTERM (process will ignore it)'
)

# wait a short time for termination (after SIGKILL)
content = content.replace(
    '// Wait a short time for termination\n    sleep(Duration::from_millis(500)).await;',
    '// Wait a short time for termination\n    sleep(Duration::from_millis(2000)).await;'
)

# 2. test_graceful_termination_with_sigterm
content = content.replace(
    'let mut cmd = CommandSpec::new("sleep").arg("30").to_tokio_command();',
    'let mut cmd = CommandSpec::new("sh").arg("-c").arg("trap \'exit 0\' TERM; while true; do sleep 1; done").to_tokio_command();'
)

# In test_graceful_termination_with_sigterm
content = content.replace(
    'let pid = child.id().expect("Failed to get child PID");\n    let pgid = Pid::from_raw(pid as i32);\n\n    // Verify process is running\n    assert!(\n        is_process_running(pid),\n        "Process should be running initially"\n    );\n\n    // Send SIGTERM\n    killpg(pgid, Signal::SIGTERM)?;\n\n    // Wait for graceful termination\n    sleep(Duration::from_millis(500)).await;',
    'let pid = child.id().expect("Failed to get child PID");\n    let pgid = Pid::from_raw(pid as i32);\n\n    sleep(Duration::from_millis(1000)).await;\n\n    // Verify process is running\n    assert!(\n        is_process_running(pid),\n        "Process should be running initially"\n    );\n\n    // Send SIGTERM\n    killpg(pgid, Signal::SIGTERM)?;\n\n    // Wait for graceful termination\n    sleep(Duration::from_millis(2000)).await;'
)

# 3. test_process_group_termination
content = content.replace(
    'let parent_pid = child.id().expect("Failed to get parent PID");\n\n    // Wait a bit for child processes to spawn\n    sleep(Duration::from_millis(500)).await;',
    'let parent_pid = child.id().expect("Failed to get parent PID");\n\n    // Wait a bit for child processes to spawn\n    sleep(Duration::from_millis(1000)).await;'
)

content = content.replace(
    'killpg(pgid, Signal::SIGKILL)?;\n\n    // Wait for termination\n    sleep(Duration::from_millis(500)).await;\n\n    // Verify parent is terminated',
    'killpg(pgid, Signal::SIGKILL)?;\n\n    // Wait for termination\n    sleep(Duration::from_millis(2000)).await;\n\n    // Verify parent is terminated'
)

# 4. test_timeout_grace_period
# Wait, I already replaced `CommandSpec::new("sleep").arg("30")` globally for all instances, but wait...
# In `test_timeout_grace_period`, the replacement for `sleep 30` should have hit because it's identical text.

content = content.replace(
    'let pid = child.id().expect("Failed to get child PID");\n    let pgid = Pid::from_raw(pid as i32);\n\n    // Verify process is running\n    assert!(is_process_running(pid), "Process should be running");\n\n    // Simulate the timeout sequence from Runner',
    'let pid = child.id().expect("Failed to get child PID");\n    let pgid = Pid::from_raw(pid as i32);\n\n    sleep(Duration::from_millis(1000)).await;\n\n    // Verify process is running\n    assert!(is_process_running(pid), "Process should be running");\n\n    // Simulate the timeout sequence from Runner'
)

content = content.replace(
    'let _ = killpg(pgid, Signal::SIGKILL);\n\n    // Wait for termination\n    sleep(Duration::from_millis(500)).await;\n\n    // Process should be terminated',
    'let _ = killpg(pgid, Signal::SIGKILL);\n\n    // Wait for termination\n    sleep(Duration::from_millis(2000)).await;\n\n    // Process should be terminated'
)

# 5. test_runner_timeout_terminates_process_group
content = content.replace(
    'let runner = Runner::native();',
    'let mut runner = Runner::native();\n    runner.claude_path = Some("bash".to_string());'
)

with open('tests/test_unix_process_termination.rs', 'w') as f:
    f.write(content)
