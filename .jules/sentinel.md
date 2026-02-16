## 2025-02-15 - Orphan Process Cleanup in NativeRunner
**Vulnerability:** NativeRunner processes on Unix could leave orphaned child processes running after timeout/termination if they spawned subprocesses (e.g., shell scripts with `&`).
**Learning:** `std::process::Command` kills only the direct child process. To kill the entire process tree, we must use process groups (`setpgid(0, 0)`) and kill the group (`kill(-pid, ...)`).
**Prevention:** Always use process groups when spawning processes that might spawn children, especially in runner/task execution contexts.
