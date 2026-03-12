#!/bin/bash
cat << 'INNER_EOF' > tests/test_unix_process_termination.rs.patch
--- tests/test_unix_process_termination.rs
+++ tests/test_unix_process_termination.rs
@@ -118,9 +118,11 @@ async fn test_sigterm_then_sigkill_sequence() -> Result<()> {
     use nix::unistd::Pid;

     // Spawn a process that ignores SIGTERM (to test SIGKILL)
+    // We use a while loop to ensure 'sh' does not optimize via exec,
+    // which would replace 'sh' with 'sleep' and lose the trap handler.
     let mut cmd = CommandSpec::new("sh")
         .arg("-c")
-        .arg("trap '' TERM; sleep 30") // Ignore SIGTERM, sleep for 30 seconds
+        .arg("trap '' TERM; while true; do sleep 1; done") // Ignore SIGTERM, run forever
         .to_tokio_command();
     cmd.stdin(Stdio::null())
         .stdout(Stdio::null())
@@ -141,12 +143,11 @@ async fn test_sigterm_then_sigkill_sequence() -> Result<()> {
     let pid = child.id().expect("Failed to get child PID");
     let pgid = Pid::from_raw(pid as i32);

+    // Wait a brief moment for the shell to start up and install its trap handler
+    sleep(Duration::from_millis(500)).await;
+
     // Verify process is running
-    assert!(
-        is_process_running(pid),
-        "Process should be running initially"
-    );
+    assert!(child.try_wait().unwrap().is_none(), "Process should be running initially");

     // Send SIGTERM (process will ignore it)
     killpg(pgid, Signal::SIGTERM)?;
@@ -155,10 +156,7 @@ async fn test_sigterm_then_sigkill_sequence() -> Result<()> {
     sleep(Duration::from_millis(500)).await;

     // Process should still be running (it ignored SIGTERM)
-    assert!(
-        is_process_running(pid),
-        "Process should still be running after SIGTERM"
-    );
+    assert!(child.try_wait().unwrap().is_none(), "Process should still be running after SIGTERM");

     // Send SIGKILL (cannot be ignored)
     killpg(pgid, Signal::SIGKILL)?;
@@ -167,10 +165,7 @@ async fn test_sigterm_then_sigkill_sequence() -> Result<()> {
-    sleep(Duration::from_millis(500)).await;
+    sleep(Duration::from_millis(1000)).await;

     // Process should now be terminated
-    assert!(
-        !is_process_running(pid),
-        "Process should be terminated after SIGKILL"
-    );
+    assert!(child.try_wait().unwrap().is_some(), "Process should be terminated after SIGKILL");

     // Clean up
     let _ = child.wait().await;
@@ -218,25 +213,19 @@ async fn test_graceful_termination_with_sigterm() -> Result<()> {
     let pgid = Pid::from_raw(pid as i32);

     // Verify process is running
-    assert!(
-        is_process_running(pid),
-        "Process should be running initially"
-    );
+    assert!(child.try_wait().unwrap().is_none(), "Process should be running initially");

     // Send SIGTERM
     killpg(pgid, Signal::SIGTERM)?;

     // Wait for graceful termination
-    sleep(Duration::from_millis(500)).await;
+    sleep(Duration::from_millis(1000)).await;

     // Process should be terminated (sleep responds to SIGTERM)
-    assert!(
-        !is_process_running(pid),
-        "Process should be terminated after SIGTERM"
-    );
+    assert!(child.try_wait().unwrap().is_some(), "Process should be terminated after SIGTERM");

     // Clean up
     let _ = child.wait().await;
@@ -284,25 +273,19 @@ async fn test_process_group_termination() -> Result<()> {
     sleep(Duration::from_millis(500)).await;

     // Verify parent is running
-    assert!(
-        is_process_running(parent_pid),
-        "Parent process should be running"
-    );
+    assert!(child.try_wait().unwrap().is_none(), "Parent process should be running");

     // Terminate the entire process group
     use nix::sys::signal::{Signal, killpg};
     use nix::unistd::Pid;
     let pgid = Pid::from_raw(parent_pid as i32);
     killpg(pgid, Signal::SIGKILL)?;

     // Wait for termination
-    sleep(Duration::from_millis(500)).await;
+    sleep(Duration::from_millis(1000)).await;

     // Verify parent is terminated
-    assert!(
-        !is_process_running(parent_pid),
-        "Parent process should be terminated"
-    );
+    assert!(child.try_wait().unwrap().is_some(), "Parent process should be terminated");

     // Clean up
     let _ = child.wait().await;
@@ -328,6 +311,8 @@ async fn test_runner_timeout_terminates_process_group() -> Result<()> {

     // Create a runner with a short timeout
     let runner = Runner::native();
+    let mut runner = runner;
+    runner.wsl_options.claude_path = Some("bash".to_string());

     // Execute with a very short timeout (1 second)
     let timeout_duration = Some(Duration::from_secs(1));
@@ -392,7 +377,7 @@ async fn test_timeout_grace_period() -> Result<()> {
     let pgid = Pid::from_raw(pid as i32);

     // Verify process is running
-    assert!(is_process_running(pid), "Process should be running");
+    assert!(child.try_wait().unwrap().is_none(), "Process should be running");

     // Simulate the timeout sequence from Runner
     // 1. Send SIGTERM
@@ -414,13 +399,10 @@ async fn test_timeout_grace_period() -> Result<()> {
     let _ = killpg(pgid, Signal::SIGKILL);

     // Wait for termination
-    sleep(Duration::from_millis(500)).await;
+    sleep(Duration::from_millis(1000)).await;

     // Process should be terminated
-    assert!(
-        !is_process_running(pid),
-        "Process should be terminated after SIGKILL"
-    );
+    assert!(child.try_wait().unwrap().is_some(), "Process should be terminated after SIGKILL");

     // Clean up
     let _ = child.wait().await;
INNER_EOF
patch -p0 < tests/test_unix_process_termination.rs.patch
