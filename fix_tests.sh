#!/bin/bash
git restore tests/test_unix_process_termination.rs
sed -i 's/sleep(Duration::from_millis(500)).await;/sleep(Duration::from_millis(1000)).await;/g' tests/test_unix_process_termination.rs
sed -i "s/\.arg(\"trap '' TERM; sleep 30\")/.arg(\"trap '' TERM; while true; do sleep 1; done\")/g" tests/test_unix_process_termination.rs

# Fix test_runner_timeout_terminates_process_group mock claude error
sed -i 's/let runner = Runner::native();/let mut runner = Runner::native();\n    runner.wsl_options.claude_path = Some("bash".to_string());/g' tests/test_unix_process_termination.rs
