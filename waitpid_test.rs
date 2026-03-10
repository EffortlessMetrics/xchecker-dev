fn main() {
    let pid = 1234;
    use nix::sys::wait::{waitpid, WaitPidFlag};
    use nix::unistd::Pid;

    let wpid = Pid::from_raw(pid as i32);
    let res = waitpid(wpid, Some(WaitPidFlag::WNOHANG));
    println!("{:?}", res);
}
