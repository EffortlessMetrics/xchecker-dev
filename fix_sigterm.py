import re

with open('tests/test_unix_process_termination.rs', 'r') as f:
    content = f.read()

# Replace the command spec for test_sigterm_then_sigkill_sequence
# It might need a stronger ignore implementation that isn't killed by SIGTERM, e.g. trapping in a loop.
old_cmd = 'let mut cmd = CommandSpec::new("sh")\n        .arg("-c")\n        .arg("trap \'\' TERM; sleep 30") // Ignore SIGTERM, sleep for 30 seconds\n        .to_tokio_command();'
new_cmd = 'let mut cmd = CommandSpec::new("sh")\n        .arg("-c")\n        .arg("trap \'\' TERM; while true; do sleep 1; done")\n        .to_tokio_command();'
content = content.replace(old_cmd, new_cmd)

# remove is_process_running completely since it's unused
content = re.sub(r'/// Check if a process is still running\nfn is_process_running\(pid: u32\) -> bool \{[\s\S]*?\}', '', content)


with open('tests/test_unix_process_termination.rs', 'w') as f:
    f.write(content)
