#!/bin/bash
sed -i 's/runner = Runner::native()/runner = Runner::native();\n    let mut runner = runner;\n    runner.wsl_options.claude_path = Some("bash".to_string());/g' tests/test_unix_process_termination.rs
