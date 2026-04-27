sed -i 's/sleep {}/sh -c "while true; do sleep 1; done"/g' tests/test_unix_process_termination.rs
sed -i 's/CommandSpec::new("sleep").arg("30")/CommandSpec::new("sh").arg("-c").arg("while true; do sleep 1; done")/g' tests/test_unix_process_termination.rs
sed -i 's/duration_secs, duration_secs, duration_secs//g' tests/test_unix_process_termination.rs
sed -i 's/duration_secs: u64/_duration_secs: u64/g' tests/test_unix_process_termination.rs
