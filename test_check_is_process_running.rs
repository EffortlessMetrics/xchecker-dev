fn is_process_running(pid: u32) -> bool {
    use nix::sys::signal::kill;
    use nix::unistd::Pid;

    let pid = Pid::from_raw(pid as i32);
    // Signal 0 (None) doesn't send a signal but checks if the process exists
    kill(pid, None).is_ok()
}

fn main() {
    let pid = std::process::id();
    println!("running: {}", is_process_running(pid));
}
