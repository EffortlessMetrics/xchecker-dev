#!/bin/bash
sed -i "s/let runner = Runner::native();/let mut runner = Runner::native(); runner.wsl_options.claude_path = Some(\"bash\".to_string());/g" tests/test_unix_process_termination.rs
