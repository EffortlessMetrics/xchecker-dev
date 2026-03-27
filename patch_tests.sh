#!/bin/bash
sed -i 's/assert!(is_process_running(pid), "Process should be running");/assert!(child.try_wait().unwrap().is_none(), "Process should be running");/g' tests/test_unix_process_termination.rs
sed -i 's/assert!(!is_process_running(pid), "Process should be terminated/assert!(child.try_wait().unwrap().is_some(), "Process should be terminated/g' tests/test_unix_process_termination.rs
