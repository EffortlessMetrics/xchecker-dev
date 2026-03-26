## 2024-03-26 - Prevent Zombie Processes via kill_on_drop
**Vulnerability:** Resource Leak (CWE-404) leading to potential Denial of Service (DoS). When using `tokio::time::timeout` with `tokio::process::Command`, the process is not automatically terminated when the timeout future is dropped.
**Learning:** Dropping the timeout future simply drops the `Child` struct, but does not send a `SIGKILL` to the actual OS process by default. This leads to leaked zombie processes over time, especially during flaky executions or malicious long-running inputs.
**Prevention:** Always append `.kill_on_drop(true)` when building a `tokio::process::Command` if it might be executed inside a timeout or dropped early.
