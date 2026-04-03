with open("tests/test_unix_process_termination.rs", "r") as f:
    content = f.read()

content = content.replace('let mut child = cmd.spawn()?;', 'let mut child = cmd.spawn()?;\n    sleep(Duration::from_millis(1000)).await; // Allow handler registration')
content = content.replace('sleep(Duration::from_millis(500)).await;', 'sleep(Duration::from_millis(1500)).await;')

with open("tests/test_unix_process_termination.rs", "w") as f:
    f.write(content)
