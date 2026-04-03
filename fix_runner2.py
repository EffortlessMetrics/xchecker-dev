with open("tests/test_unix_process_termination.rs", "r") as f:
    content = f.read()

content = content.replace("let runner = Runner::native();", 'let mut runner = Runner::native();\n    runner.wsl_options.claude_path = Some("bash".to_string());')
content = content.replace('let mut cmd = CommandSpec::new("sleep").arg("10").to_tokio_command();', 'let mut cmd = CommandSpec::new("sh").arg("-c").arg("trap \'\' TERM; while true; do sleep 1; done").to_tokio_command();\ncmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());')

with open("tests/test_unix_process_termination.rs", "w") as f:
    f.write(content)
