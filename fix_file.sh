#!/bin/bash
sed -i 's/runner = Runner::native();;$/runner = Runner::native();/g' tests/test_unix_process_termination.rs
sed -i 's/Some("bash".to_string());;/Some("bash".to_string());/g' tests/test_unix_process_termination.rs
