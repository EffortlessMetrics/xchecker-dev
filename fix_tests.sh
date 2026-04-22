#!/bin/bash

# Fix test_sigterm_then_sigkill_sequence
sed -i "s/trap '' TERM; sleep 30/trap '' TERM; while true; do sleep 1; done/g" tests/test_unix_process_termination.rs

# Fix test_graceful_termination_with_sigterm
sed -i "s/CommandSpec::new(\"sleep\").arg(\"30\").to_tokio_command()/CommandSpec::new(\"sh\").arg(\"-c\").arg(\"trap 'exit 0' TERM; while true; do sleep 1; done\").to_tokio_command()/g" tests/test_unix_process_termination.rs

# Fix test_timeout_grace_period
sed -i "s/CommandSpec::new(\"sleep\").arg(\"30\").to_tokio_command()/CommandSpec::new(\"sh\").arg(\"-c\").arg(\"trap '' TERM; while true; do sleep 1; done\").to_tokio_command()/g" tests/test_unix_process_termination.rs

# Add sleep 1000ms after spawn
sed -i '/let pgid = Pid::from_raw(pid as i32);/a \
\
    // Allow signal handler registration\
    tokio::time::sleep(Duration::from_millis(1000)).await;' tests/test_unix_process_termination.rs

# Add sleep 1000ms after script spawn
sed -i '/let parent_pid = child.id().expect("Failed to get parent PID");/a \
\
    // Allow script to start spawning children\
    tokio::time::sleep(Duration::from_millis(1000)).await;' tests/test_unix_process_termination.rs

# Fix test_runner_timeout_terminates_process_group
sed -i '/runner.options.claude_path = Some("bash".to_string());/a \
    runner.wsl_options.claude_path = Some("bash".to_string());' tests/test_unix_process_termination.rs
