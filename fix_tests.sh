sed -i 's/sleep(Duration::from_millis(2000)).await;/sleep(Duration::from_millis(5000)).await;/g' tests/test_unix_process_termination.rs
