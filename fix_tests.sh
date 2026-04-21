sed -i 's/let mut child = cmd.spawn()?;/let mut child = cmd.spawn()?; tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;/g' tests/test_unix_process_termination.rs
sed -i 's/sleep(Duration::from_millis(500)).await;/sleep(Duration::from_millis(2000)).await;/g' tests/test_unix_process_termination.rs
sed -i 's/let runner = Runner::native();/let mut runner = Runner::native(); runner.wsl_options.claude_path = Some("bash".to_string());/g' tests/test_unix_process_termination.rs
sed -i 's/sleep 30/while true; do sleep 1; done/g' tests/test_unix_process_termination.rs
