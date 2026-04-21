sed -i 's/let runner = Runner::native();/let mut runner = Runner::native();\n    runner.wsl_options.claude_path = Some("bash".to_string());/g' tests/test_unix_process_termination.rs
sed -i 's/cmd.spawn()/sleep(Duration::from_millis(1000)).await;\n    let mut child = cmd.spawn()/g' tests/test_unix_process_termination.rs
sed -i 's/sleep 30/while true; do sleep 1; done/g' tests/test_unix_process_termination.rs
