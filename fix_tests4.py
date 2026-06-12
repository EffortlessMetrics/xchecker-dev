# If it's a zombie, `kill(pid, 0)` is successful (returns OK)!
# So the process IS running as far as `is_process_running` is concerned, until `wait` is called!
# But wait, in the original code, `assert!(!is_process_running(pid))` was called BEFORE `let _ = child.wait().await;`.
# If `child.wait().await` is called before, it will block if it hasn't exited, OR clean up the zombie.
# Let's replace `sleep(Duration::from_millis(2000)).await;` with `let _ = child.wait().await;` BEFORE `assert!(!is_process_running)`. Wait, no, we can't always do that, or maybe we can? But the original code had:
# `sleep(500); assert!(!is_process_running(pid)); let _ = child.wait().await;`
# Why did it ever pass before? Maybe `sleep 30` wasn't a zombie because ... wait, no.
# Actually, tokio's `Child` automatically reaps zombies in the background when it terminates! Tokio's `Child` uses a background signal handler (`SIGCHLD`) to `waitpid`!
# So tokio reaps it, and `is_process_running` returns false!

# So why did it fail now? "timing-dependent signal handling" means it's flaky in CI.
# Did I not add `sleep(Duration::from_millis(2000)).await;` correctly?
