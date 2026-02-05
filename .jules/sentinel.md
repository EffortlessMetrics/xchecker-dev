
## 2025-05-24 - [Process Leak Fix in NativeRunner]
**Vulnerability:** `NativeRunner` on Unix only terminated the direct child process on timeout, leaving any grandchildren (e.g. from shell scripts) running as orphans (resource leak).
**Learning:** `std::process::Command` by default keeps the child in the same process group or a new one depending on invocation, but to reliably kill a tree, we must explicitly set a new process group and kill the group.
**Prevention:** Use `CommandExt::process_group(0)` to isolate the child and `kill(-(pid), SIGKILL)` to terminate the entire group.

## 2025-05-24 - [Unreliable Signal Testing with Shell]
**Vulnerability:** Tests relying on shell `trap` to verify signal handling in process groups were flaky because shell behavior with signals and child processes is complex and timing-dependent.
**Learning:** Shells may exit if a child process dies from a signal even if the shell trapped that signal, or trap handling may be delayed.
**Prevention:** Use `perl` or another language with direct signal handling support for testing process termination logic. Explicitly synchronize initialization using a "READY" marker on stdout before sending signals.
