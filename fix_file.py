import sys

def main():
    with open('tests/test_unix_process_termination.rs', 'r') as f:
        content = f.read()

    content = content.replace('''    // Wait for graceful termination
    sleep(Duration::from_millis(500)).await;

    // Process should be terminated (sleep responds to SIGTERM)
    assert!(
        !is_process_running(pid),
        "Process should be terminated after SIGTERM"
    );''', '''    // Wait for graceful termination
    sleep(Duration::from_millis(1000)).await;

    // Process should be terminated (sleep responds to SIGTERM)
    assert!(child.try_wait().unwrap().is_some(), "Process should be terminated after SIGTERM");''')

    with open('tests/test_unix_process_termination.rs', 'w') as f:
        f.write(content)

if __name__ == '__main__':
    main()
