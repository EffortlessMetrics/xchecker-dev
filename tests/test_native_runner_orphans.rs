use std::thread;
use std::time::Duration;
use xchecker::runner::{CommandSpec, NativeRunner, ProcessRunner};

#[test]
#[cfg(unix)]
fn test_native_runner_orphans_and_isolation() {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    // Script to:
    // 1. Write its own PGID to file.
    // 2. Spawn a child that sleeps.
    // 3. Write child PID to file.
    // 4. Wait.
    let script_content = r#"#!/bin/sh
# Write PGID to file
ps -o pgid= -p $$ > child_pgid.txt

# Spawn child
sleep 10 &
echo $! > grandchild_pid.txt
wait
"#;

    let script_path = "test_orphan_isolation.sh";
    let child_pgid_file = "child_pgid.txt";
    let grandchild_pid_file = "grandchild_pid.txt";

    // Cleanup previous run if any
    let _ = fs::remove_file(script_path);
    let _ = fs::remove_file(child_pgid_file);
    let _ = fs::remove_file(grandchild_pid_file);

    fs::write(script_path, script_content).unwrap();
    fs::set_permissions(script_path, fs::Permissions::from_mode(0o755)).unwrap();

    let runner = NativeRunner::new();
    let cmd = CommandSpec::new("./test_orphan_isolation.sh");

    println!("Starting runner with short timeout...");
    // Run with short timeout to force termination
    // We don't care about result, it should be Timeout.
    let _ = runner.run(&cmd, Duration::from_millis(500));

    // Give some time for cleanup
    thread::sleep(Duration::from_millis(100));

    // 1. Check PGID isolation
    if let Ok(pgid_str) = fs::read_to_string(child_pgid_file) {
        let child_pgid: i32 = pgid_str.trim().parse().unwrap();
        let my_pgid = unsafe { libc::getpgid(0) };
        println!("My PGID: {}, Child PGID: {}", my_pgid, child_pgid);

        // Assert that child is in a NEW process group
        // This will fail if NativeRunner doesn't use setpgid(0, 0)
        assert_ne!(
            child_pgid, my_pgid,
            "Child process should be in a new process group (isolation failure)"
        );
    } else {
        panic!("Could not read child_pgid.txt - script failed to run?");
    }

    // 2. Check Orphan cleanup
    if let Ok(pid_str) = fs::read_to_string(grandchild_pid_file) {
        let pid_val: i32 = pid_str.trim().parse().unwrap();

        // Check if process exists using kill(pid, 0)
        let kill_res = unsafe { libc::kill(pid_val, 0) };
        let running = kill_res == 0;
        println!(
            "Grandchild PID: {}, kill(0) result: {}, running: {}",
            pid_val, kill_res, running
        );

        // If running, kill it to clean up test mess
        if running {
            unsafe { libc::kill(pid_val, libc::SIGKILL) };
        }

        // Assert that grandchild is NOT running
        // This will fail if NativeRunner kills PID instead of PGID
        assert!(
            !running,
            "Grandchild process should have been terminated (orphan cleanup failure)"
        );
    } else {
        panic!("Could not read grandchild_pid.txt - script failed to reach spawn?");
    }

    // Cleanup
    let _ = fs::remove_file(script_path);
    let _ = fs::remove_file(child_pgid_file);
    let _ = fs::remove_file(grandchild_pid_file);
}
