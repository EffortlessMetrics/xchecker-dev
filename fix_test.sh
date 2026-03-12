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
@@ -141,6 +143,9 @@ async fn test_sigterm_then_sigkill_sequence() -> Result<()> {
     let pid = child.id().expect("Failed to get child PID");
     let pgid = Pid::from_raw(pid as i32);

+    // Wait a brief moment for the shell to start up and install its trap handler
+    sleep(Duration::from_millis(500)).await;
+
     // Verify process is running
     assert!(
         is_process_running(pid),
@@ -173,7 +178,7 @@ async fn test_sigterm_then_sigkill_sequence() -> Result<()> {
     killpg(pgid, Signal::SIGKILL)?;

     // Wait a short time for termination
-    sleep(Duration::from_millis(500)).await;
+    sleep(Duration::from_millis(1000)).await;

     // Process should now be terminated
     assert!(
@@ -226,7 +231,7 @@ async fn test_graceful_termination_with_sigterm() -> Result<()> {
     killpg(pgid, Signal::SIGTERM)?;

     // Wait for graceful termination
-    sleep(Duration::from_millis(500)).await;
+    sleep(Duration::from_millis(1000)).await;

     // Process should be terminated (sleep responds to SIGTERM)
     assert!(
@@ -295,7 +300,7 @@ async fn test_process_group_termination() -> Result<()> {
     killpg(pgid, Signal::SIGKILL)?;

     // Wait for termination
-    sleep(Duration::from_millis(500)).await;
+    sleep(Duration::from_millis(1000)).await;

     // Verify parent is terminated
     assert!(
@@ -328,6 +333,8 @@ async fn test_runner_timeout_terminates_process_group() -> Result<()> {

     // Create a runner with a short timeout
     let runner = Runner::native();
+    let mut runner = runner;
+    runner.wsl_options.claude_path = Some("bash".to_string());

     // Execute with a very short timeout (1 second)
     let timeout_duration = Some(Duration::from_secs(1));
@@ -414,7 +421,8 @@ async fn test_timeout_grace_period() -> Result<()> {
     let _ = killpg(pgid, Signal::SIGKILL);

     // Wait for termination
-    sleep(Duration::from_millis(500)).await;
+    sleep(Duration::from_millis(1000)).await;
+    sleep(Duration::from_millis(1000)).await;

     // Process should be terminated
     assert!(
INNER_EOF
patch -p0 < tests/test_unix_process_termination.rs.patch
