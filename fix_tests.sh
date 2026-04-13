#!/bin/bash

# Fix test_runner_timeout_terminates_process_group mock claude error
sed -i 's/let runner = Runner::native();/let mut runner = Runner::native();\n    runner.wsl_options.claude_path = Some("bash".to_string());/g' tests/test_unix_process_termination.rs
