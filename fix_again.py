import re

with open('tests/test_unix_process_termination.rs', 'r') as f:
    content = f.read()

# Add a sleep to the beginning of the script so it has time to set up the trap
content = content.replace('trap \'\' TERM; while true; do sleep 1; done', 'sleep 1; trap \'\' TERM; while true; do sleep 1; done')
content = content.replace('sleep(Duration::from_millis(1000)).await;', 'sleep(Duration::from_millis(2000)).await;')

with open('tests/test_unix_process_termination.rs', 'w') as f:
    f.write(content)
