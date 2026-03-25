import re

with open('tests/test_unix_process_termination.rs', 'r') as f:
    content = f.read()

# Add a sleep right after spawn, before sending signals, to give the shell time to start and register the trap.
old_str = "let mut child = cmd.spawn()?;"
new_str = "let mut child = cmd.spawn()?;\n    sleep(Duration::from_millis(1000)).await;"
content = content.replace(old_str, new_str)

with open('tests/test_unix_process_termination.rs', 'w') as f:
    f.write(content)
