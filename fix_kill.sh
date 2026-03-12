#!/bin/bash
cat << 'INNER_EOF' > tests/test_unix_process_termination.rs.patch
--- tests/test_unix_process_termination.rs
+++ tests/test_unix_process_termination.rs
@@ -35,7 +35,13 @@

     let pid = Pid::from_raw(pid as i32);
     // Signal 0 (None) doesn't send a signal but checks if the process exists
-    kill(pid, None).is_ok()
+    match kill(pid, None) {
+        Ok(_) => true,
+        Err(nix::errno::Errno::ESRCH) => false,
+        Err(nix::errno::Errno::EPERM) => true,
+        // Other errors just mean the process might be dead but not reaped
+        Err(_) => false,
+    }
 }

 /// Create a test script that spawns child processes
INNER_EOF
patch -p0 < tests/test_unix_process_termination.rs.patch
