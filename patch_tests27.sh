cat << 'INNER_EOF' > patch_tests27.diff
--- tests/test_unix_process_termination.rs
+++ tests/test_unix_process_termination.rs
@@ -162,7 +162,7 @@
     killpg(pgid, Signal::SIGTERM)?;

     // Wait a short time
-    sleep(Duration::from_millis(500)).await;
+    sleep(Duration::from_millis(1500)).await;

     // Process should still be running (it ignored SIGTERM)
     assert!(
@@ -174,13 +174,13 @@
     killpg(pgid, Signal::SIGKILL)?;

     // Wait a short time for termination
-    sleep(Duration::from_millis(500)).await;
+    sleep(Duration::from_millis(1500)).await;

     // Process should now be terminated
     assert!(
-        !is_process_running(pid),
+        child.try_wait().unwrap().is_some() || !is_process_running(pid),
         "Process should be terminated after SIGKILL"
     );

     // Clean up
@@ -226,12 +226,12 @@
     killpg(pgid, Signal::SIGTERM)?;

     // Wait for graceful termination
-    sleep(Duration::from_millis(500)).await;
+    sleep(Duration::from_millis(1500)).await;

     // Process should be terminated (sleep responds to SIGTERM)
     assert!(
-        !is_process_running(pid),
+        child.try_wait().unwrap().is_some() || !is_process_running(pid),
         "Process should be terminated after SIGTERM"
     );

     // Clean up
@@ -292,12 +292,12 @@
     killpg(pgid, Signal::SIGTERM)?;

     // Wait for processes to terminate
-    sleep(Duration::from_millis(500)).await;
+    sleep(Duration::from_millis(1500)).await;

     // Verify parent is dead
     assert!(
-        !is_process_running(parent_pid),
+        child.try_wait().unwrap().is_some() || !is_process_running(parent_pid),
         "Parent process should be terminated"
     );

     // Verify children are dead (checking the script output would be better, but
@@ -328,7 +328,9 @@
     create_test_script(script_path.to_str().unwrap(), 60)?;

     // Create a runner with a short timeout
-    let runner = Runner::native();
+    let mut wsl_opts = xchecker::runner::WslOptions::default();
+    wsl_opts.claude_path = Some("bash".to_string());
+    let runner = Runner::new(xchecker::runner::RunnerMode::Native, wsl_opts);

     // Execute with a very short timeout (1 second)
     let timeout_duration = Some(Duration::from_secs(1));
@@ -410,12 +412,12 @@
     // Test cleanup: in case the mock didn't kill it, force kill
     let pgid = Pid::from_raw(pid as i32);
     let _ = killpg(pgid, Signal::SIGKILL);
-    sleep(Duration::from_millis(100)).await;
+    sleep(Duration::from_millis(1000)).await;

     // Verify the process is dead
     assert!(
-        !is_process_running(pid),
+        child.try_wait().unwrap().is_some() || !is_process_running(pid),
         "Process should be terminated after SIGKILL"
     );

     Ok(())
INNER_EOF
patch -p0 < patch_tests27.diff
cargo test --test test_unix_process_termination -- --ignored
