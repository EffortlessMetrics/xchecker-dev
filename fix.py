with open("tests/test_unix_process_termination.rs", "r") as f:
    content = f.read()

import re

# Update is_process_running to take child instead of pid
content = re.sub(
    r'fn is_process_running\(pid: u32\) -> bool \{[^\}]+\}',
    'fn is_process_running_via_child(child: &mut tokio::process::Child) -> bool {\n    child.try_wait().unwrap().is_none()\n}',
    content
)

content = content.replace("is_process_running(pid)", "is_process_running_via_child(&mut child)")
content = content.replace("is_process_running(parent_pid)", "is_process_running_via_child(&mut child)")

with open("tests/test_unix_process_termination.rs", "w") as f:
    f.write(content)
