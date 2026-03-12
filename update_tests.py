import sys

def main():
    with open('tests/test_unix_process_termination.rs', 'r') as f:
        content = f.read()

    # Replace wait times
    content = content.replace('sleep(Duration::from_millis(500)).await;', 'sleep(Duration::from_millis(1000)).await;')

    # Replace runner init
    content = content.replace('let runner = Runner::native();', 'let mut runner = Runner::native();\n    runner.wsl_options.claude_path = Some("bash".to_string());')

    # Replace sh trap
    content = content.replace('arg("trap \'\' TERM; sleep 30")', 'arg("trap \'\' TERM; while true; do sleep 1; done")')

    with open('tests/test_unix_process_termination.rs', 'w') as f:
        f.write(content)

if __name__ == '__main__':
    main()
