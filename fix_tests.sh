sed -i 's/let runner = Runner::native();/let mut runner = Runner::native();\n    runner.wsl_options.claude_path = Some("bash".to_string());/g' tests/test_unix_process_termination.rs
sed -i 's/let mut child = cmd.spawn()?;/let mut child = cmd.spawn()?;\n    sleep(Duration::from_millis(1000)).await;/g' tests/test_unix_process_termination.rs
sed -i 's/CommandSpec::new("sh")/CommandSpec::new("bash")/g' tests/test_unix_process_termination.rs
sed -i 's/killpg(pgid, Signal::SIGTERM)?;/killpg(pgid, Signal::SIGTERM)?;\n    let _ = child.kill().await;\n    let _ = child.wait().await;/g' tests/test_unix_process_termination.rs
sed -i 's/CommandSpec::new("sleep").arg("30")/CommandSpec::new("bash").arg("-c").arg("while true; do sleep 1; done")/g' tests/test_unix_process_termination.rs
sed -i 's/let _ = killpg(pgid, Signal::SIGKILL);/let _ = killpg(pgid, Signal::SIGKILL);\n    let _ = child.kill().await;\n    let _ = child.wait().await;/g' tests/test_unix_process_termination.rs
sed -i 's/killpg(pgid, Signal::SIGKILL)?;/killpg(pgid, Signal::SIGKILL)?;\n    let _ = child.kill().await;\n    let _ = child.wait().await;/g' tests/test_unix_process_termination.rs
sed -i "s/arg(\"trap '' TERM; sleep 30\")/arg(\"trap '' TERM; while true; do sleep 1; done\")/g" tests/test_unix_process_termination.rs
