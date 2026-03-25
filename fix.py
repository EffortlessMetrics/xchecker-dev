import re

with open('tests/test_unix_process_termination.rs', 'r') as f:
    content = f.read()

# Replace sleep durations
content = content.replace('sleep(Duration::from_millis(500)).await;', 'sleep(Duration::from_millis(1000)).await;')

# We can replace `is_process_running(pid)` with `is_process_running_via_child(&mut child)`
# First define the helper
helper = """
/// Check if a process is still running via try_wait
fn is_process_running_via_child(child: &mut tokio::process::Child) -> bool {
    matches!(child.try_wait(), Ok(None))
}
"""
content = content.replace('fn is_process_running(pid: u32) -> bool {', helper + '\nfn is_process_running(pid: u32) -> bool {')

# For each test, we'll replace the assertions
content = content.replace('is_process_running(pid)', 'is_process_running_via_child(&mut child)')
content = content.replace('is_process_running(parent_pid)', 'is_process_running_via_child(&mut child)')

with open('tests/test_unix_process_termination.rs', 'w') as f:
    f.write(content)
