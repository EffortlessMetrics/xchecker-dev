#!/bin/bash
sed -i 's/sleep(Duration::from_millis(500)).await/sleep(Duration::from_millis(1500)).await/g' tests/test_unix_process_termination.rs
sed -i 's/let runner = Runner::native();/let mut runner = Runner::native();/' tests/test_unix_process_termination.rs
sed -i '/let timeout_duration = Some(Duration::from_secs(1));/i \    runner.wsl_options.claude_path = Some("bash".to_string());\n' tests/test_unix_process_termination.rs
