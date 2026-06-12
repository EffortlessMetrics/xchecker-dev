use std::process::Command;
use std::os::unix::process::CommandExt;

fn main() {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg("trap 'exit 0' TERM; while true; do sleep 1; done");
    unsafe {
        cmd.pre_exec(|| {
            libc::setpgid(0, 0);
            Ok(())
        });
    }
    let mut child = cmd.spawn().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1000));
    unsafe {
        libc::killpg(child.id() as i32, libc::SIGTERM);
    }
    std::thread::sleep(std::time::Duration::from_millis(2000));
    let res = unsafe { libc::kill(child.id() as i32, 0) };
    if res == 0 {
        println!("STILL RUNNING");
    } else {
        println!("TERMINATED");
    }
    let _ = child.wait();
}
