import re

with open('tests/test_unix_process_termination.rs', 'r') as f:
    content = f.read()

# I will replace `sleep(Duration::from_millis(2000)).await;` after `killpg`
# with:
# `let _ = child.wait().await;`
# BUT wait! If it didn't terminate, `wait()` will hang the test forever!
# Actually, Tokio has `tokio::time::timeout` so we could wrap `wait()` in a timeout, or better, we can just use `try_wait()`.
# Wait, tokio's `Child` has `try_wait()`. If it returns `Ok(Some(status))`, it's terminated. If `Ok(None)`, it's running.
# The memory says: "and an increased sleep(Duration::from_millis(2000)).await after killpg to prevent race conditions during CI verification checks."
# So I should leave the `sleep` there, but we need to reap the zombie.
# I'll add `let _ = child.try_wait();` BEFORE `assert!(!is_process_running(pid))`. Wait!
# Actually, tokio's `Child` background task reaps it, but maybe we need to yield to the executor or maybe it's not fast enough?
# Wait! Tokio's `try_wait()` will reap it!
# I will do:
# `sleep(Duration::from_millis(2000)).await; let _ = child.try_wait();`
# Or better, just modify `is_process_running`? No, I shouldn't touch the rest of the file if I can just fix the tests.
# Let's add `let _ = child.try_wait();` after the `sleep 2000` in the 4 failing tests.

content = content.replace(
    'sleep(Duration::from_millis(2000)).await;\n\n    // Process should',
    'sleep(Duration::from_millis(2000)).await;\n    let _ = child.try_wait();\n\n    // Process should'
)

content = content.replace(
    'sleep(Duration::from_millis(2000)).await;\n\n    // Verify parent is terminated',
    'sleep(Duration::from_millis(2000)).await;\n    let _ = child.try_wait();\n\n    // Verify parent is terminated'
)

with open('tests/test_unix_process_termination.rs', 'w') as f:
    f.write(content)
