import re

with open('tests/test_unix_process_termination.rs', 'r') as f:
    content = f.read()

# Instead of `sleep 1; trap ...`, just use `sh` trap with a tight loop
# Actually, the problem is that `try_wait` returns `Ok(Some(ExitStatus))` if it exited.
# But wait, if we send SIGTERM to the process group, and we spawned `sh -c "trap ...; sleep ..."`, the shell handles the trap.
# If we used `sleep 1; trap '' TERM`, the trap isn't active during the first sleep! Let's revert that.
content = content.replace('sleep 1; trap \'\' TERM; while true; do sleep 1; done', 'trap \'\' TERM; while true; do sleep 1; done')

# Remove the unused function warning
content = content.replace('fn is_process_running(pid: u32) -> bool {', '#[allow(dead_code)]\nfn is_process_running(pid: u32) -> bool {')

with open('tests/test_unix_process_termination.rs', 'w') as f:
    f.write(content)
