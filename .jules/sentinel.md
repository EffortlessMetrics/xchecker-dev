
## 2025-05-24 - [Process Leak Fix in NativeRunner]
**Vulnerability:** `NativeRunner` on Unix only terminated the direct child process on timeout, leaving any grandchildren (e.g. from shell scripts) running as orphans (resource leak).
**Learning:** `std::process::Command` by default keeps the child in the same process group or a new one depending on invocation, but to reliably kill a tree, we must explicitly set a new process group and kill the group.
**Prevention:** Use `CommandExt::process_group(0)` to isolate the child and `kill(-(pid), SIGKILL)` to terminate the entire group.
