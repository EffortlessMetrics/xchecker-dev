#[cfg(unix)]
#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use std::time::Duration;
    use tempfile::TempDir;
    use xchecker_utils::runner::{CommandSpec, NativeRunner, ProcessRunner, RunnerError};

    #[test]
    fn test_native_runner_leaks_grandchildren() {
        // 1. Create a script that spawns a child and waits for it
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("spawn_sleep.sh");
        let marker_path = temp_dir.path().join("child_pid.txt");

        {
            let mut file = File::create(&script_path).unwrap();
            // We use a script that writes the child PID to a file so we can check it
            // 'exec sleep 100' would replace the shell, keeping the same PID, which NativeRunner WOULD kill.
            // So we need 'sleep 100 &' and then 'wait', so there are two processes.
            // The shell (parent) and sleep (child).
            let script = format!(
                r#"#!/bin/sh
sleep 100 &
child=$!
echo $child > "{}"
# Debug info
ps -o pid,pgid,ppid,comm
wait $child
"#,
                marker_path.display()
            );
            file.write_all(script.as_bytes()).unwrap();

            let mut perms = file.metadata().unwrap().permissions();
            perms.set_mode(0o755);
            file.set_permissions(perms).unwrap();
        }

        // 2. Run the script with NativeRunner and a short timeout
        let runner = NativeRunner::new();
        let cmd = CommandSpec::new(script_path.to_str().unwrap());

        let result = runner.run(&cmd, Duration::from_secs(1));

        // 3. Verify it timed out
        assert!(matches!(result, Err(RunnerError::Timeout { .. })), "Should have timed out");

        // 4. Get the child PID
        let child_pid_str = std::fs::read_to_string(&marker_path).expect("Child PID file should exist");
        let child_pid: i32 = child_pid_str.trim().parse().expect("PID should be a number");

        // 5. Check if the child process is still running
        // We poll for a bit because signal delivery and reaping is asynchronous
        let mut is_running = true;
        for _ in 0..10 {
            // kill(pid, 0) returns 0 if process exists and we have permission to signal it
            if unsafe { libc::kill(child_pid, 0) != 0 } {
                is_running = false;
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }

        if is_running {
            println!("DEBUG: Child {} is still running!", child_pid);
            // Print process info if possible
            let _ = std::process::Command::new("ps").args(["-o", "pid,pgid,ppid,comm", "-p", &child_pid.to_string()]).status();

            // Cleanup
            unsafe { libc::kill(child_pid, libc::SIGKILL) };
        }

        // 6. Assert success (process should be gone)
        assert!(!is_running, "Grandchild process should have been killed by process group termination");
    }
}
